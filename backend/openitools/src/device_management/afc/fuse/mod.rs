use futures_util::Stream;
use openitools_idevice::afc::{AfcClient, AfcFopenMode, FileDescriptor, PathInfo};
use rfuse3::{
    raw::{prelude::*, Filesystem},
    Errno, MountOptions, Result as FResult,
};
use std::{collections::HashMap, sync::Arc};
use std::{
    ffi::{c_int, OsStr},
    sync::atomic::AtomicU64,
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, SystemTime},
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{Mutex, RwLock};

use crate::device_management::afc::FuseCommand;

const TTL: Duration = Duration::from_secs(1); // 1 second
type FH = u64;
type FD = u64;

pub struct AfcFS {
    inode_map: RwLock<HashMap<u64, String>>,
    reverse_map: RwLock<HashMap<String, u64>>,

    afc: Mutex<AfcClient>,
    open_fds: Mutex<HashMap<FH, FD>>,
    next_fh: AtomicU64,

    shutdown_tx: tokio::sync::watch::Sender<()>,
}

impl AfcFS {
    pub fn new(
        files_path: Vec<String>,
        afc: AfcClient,
        shutdown_tx: tokio::sync::watch::Sender<()>,
    ) -> Self {
        let mut inode_map = HashMap::new();
        let mut reverse_map = HashMap::new();

        // give each file its own stable inode
        for (i, path) in files_path.into_iter().enumerate() {
            let ino = (i + 2) as u64; // start at inode 2
            inode_map.insert(ino, path.clone());
            reverse_map.insert(path, ino);
        }

        Self {
            inode_map: RwLock::new(inode_map),
            reverse_map: RwLock::new(reverse_map),
            afc: Mutex::new(afc),
            open_fds: Mutex::default(),
            next_fh: AtomicU64::new(1),
            shutdown_tx,
        }
    }

    /// Create a FileAttr for the root directory
    fn root_attr(&self) -> FileAttr {
        FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: SystemTime::now().into(),
            mtime: SystemTime::now().into(),
            ctime: SystemTime::now().into(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            rdev: 0,
            blksize: 4096,

            #[cfg(target_os = "macos")]
            crtime: SystemTime::now().into(),

            #[cfg(target_os = "macos")]
            flags: 0,
        }
    }
}

impl Filesystem for AfcFS {
    async fn init(&self, _req: Request) -> FResult<ReplyInit> {
        println!("Filesystem initialization");
        Ok(ReplyInit {
            max_write: std::num::NonZeroU32::new(4096).unwrap(),
        })
    }

    async fn release(
        &self,
        req: Request,
        inode: rfuse3::Inode,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
    ) -> FResult<()> {
        println!("released: {req:#?}\n\n inode: {inode}\n\n flush: {flush}\n\n lock_owner: {lock_owner}\n\n flags: {flags}");

        let afc_fd = match self.open_fds.lock().await.remove(&fh) {
            Some(fd) => fd,
            None => return Ok(()),
        };

        let mut afc = self.afc.lock().await;

        // file path is not needed to close
        unsafe {
            FileDescriptor::new(&mut afc, afc_fd, "".into())
                .close()
                .await
                .map_err(|_| libc::ENOENT)?;
        }

        // let file = self.inode_map.write().await.remove(&inode).unwrap();
        // self.reverse_map.write().await.remove(&file);
        //
        // if self.inode_map.read().await.len() == 0 {
        //     self.shutdown_tx.send(());
        // };

        Ok(())
    }

    async fn destroy(&self, _req: Request) {
        println!("Filesystem destruction");
    }

    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> FResult<ReplyEntry> {
        let name_str = name
            .to_str()
            .ok_or_else(|| Into::<Errno>::into(libc::ENOENT))?;
        println!("lookup parent={} name={name_str}", parent);

        // handle '.' and '..'
        if parent == 1 && (name_str == "." || name_str == "..") {
            let attr = self.root_attr();
            return Ok(ReplyEntry {
                ttl: TTL,
                attr,
                generation: 0,
            });
        }

        if parent != 1 {
            return Err(libc::ENOENT.into());
        }

        if let Some(&ino) = self.reverse_map.read().await.get(name_str) {
            let inode_map = self.inode_map.read().await;
            let path = inode_map
                .get(&ino)
                .ok_or_else(|| Into::<Errno>::into(libc::ENOENT))?;

            println!("getting file info for {path:#?}");

            let file_info: PathInfo = self
                .afc
                .lock()
                .await
                .get_file_info(path)
                .await
                .map_err(|e| {
                    eprintln!("get_file_info failed: {e}");
                    libc::EIO
                })?
                .into();

            println!("got info: {file_info:#?}");

            return Ok(ReplyEntry {
                ttl: TTL,
                attr: file_info.to_rfuse_file_attr(ino),
                generation: 0,
            });
        }

        Err(libc::ENOENT.into())
    }

    async fn getattr(
        &self,
        _req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> FResult<ReplyAttr> {
        println!("getattr inode={inode}");
        if inode == 1 {
            let attr = self.root_attr();
            return Ok(ReplyAttr { ttl: TTL, attr });
        }

        if let Some(path) = self.inode_map.read().await.get(&inode) {
            println!("getting attribute of {path}");
            let file_info: PathInfo = self
                .afc
                .lock()
                .await
                .get_file_info(path)
                .await
                .map_err(|e| {
                    eprintln!("get_file_info failed: {e}");
                    libc::EIO
                })?
                .into();
            println!("info: {file_info:#?}");

            return Ok(ReplyAttr {
                ttl: TTL,
                attr: file_info.to_rfuse_file_attr(inode),
            });
        }

        Err(libc::ENOENT.into())
    }

    async fn read(
        &self,
        _req: Request,
        inode: u64,
        fh: u64,
        offset: u64,
        size: u32,
    ) -> FResult<ReplyData> {
        let file_fd = *self.open_fds.lock().await.get(&fh).ok_or(libc::EBADF)?;

        let path = self
            .inode_map
            .read()
            .await
            .get(&inode)
            .cloned()
            .ok_or(libc::ENOENT)?;

        let mut afc = self.afc.lock().await;
        println!("reading inode: {inode} path={path}");

        // TODO: buffered?
        let mut remote_file = unsafe { FileDescriptor::new(&mut afc, file_fd, path) };

        let mut buf = vec![0u8; size as usize];

        if let Err(e) = remote_file.seek(std::io::SeekFrom::Start(offset)).await {
            eprintln!("seek failed: {e}");
        }

        let n = match remote_file.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("read failed: {e}");
                return Err(libc::EIO.into());
            }
        };

        buf.truncate(n);
        Ok(ReplyData { data: buf.into() })
    }

    async fn readdir<'a>(
        &'a self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: i64,
    ) -> FResult<ReplyDirectory<impl Stream<Item = FResult<DirectoryEntry>> + Send + 'a>> {
        println!("readdir parent={parent}, offset={offset}");
        if parent != 1 {
            return Err(libc::ENOENT.into());
        }

        // Build stable entries; offsets must be monotonic and stable.
        let mut entries = Vec::with_capacity(2 + self.reverse_map.read().await.len());
        entries.push(DirectoryEntry {
            inode: 1,
            offset: 1,
            kind: FileType::Directory,
            name: std::ffi::OsString::from("."),
        });
        entries.push(DirectoryEntry {
            inode: 1,
            offset: 2,
            kind: FileType::Directory,
            name: std::ffi::OsString::from(".."),
        });

        for (path, &ino) in &*self.reverse_map.read().await {
            entries.push(DirectoryEntry {
                inode: ino,
                kind: FileType::RegularFile,
                name: path.clone().into(),
                offset: ino as i64,
            });
        }

        println!("entries: {entries:#?}");

        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.offset > offset)
            .map(Ok)
            .collect();

        Ok(ReplyDirectory {
            entries: futures_util::stream::iter(filtered),
        })
    }

    async fn readdirplus<'a>(
        &'a self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> FResult<ReplyDirectoryPlus<impl Stream<Item = FResult<DirectoryEntryPlus>> + Send + 'a>>
    {
        println!("readdirplus parent={}, offset={}", parent, offset);

        if parent != 1 {
            return Err(libc::ENOENT.into());
        }

        let root_attr = self.root_attr();

        let mut entries: Vec<DirectoryEntryPlus> =
            Vec::with_capacity(2 + self.reverse_map.read().await.len());
        entries.push(DirectoryEntryPlus {
            inode: 1,
            generation: 0,
            kind: FileType::Directory,
            name: std::ffi::OsString::from("."),
            offset: 1,
            attr: root_attr.clone(),
            entry_ttl: TTL,
            attr_ttl: TTL,
        });
        entries.push(DirectoryEntryPlus {
            inode: 1,
            generation: 0,
            kind: FileType::Directory,
            name: std::ffi::OsString::from(".."),
            offset: 2,
            attr: root_attr.clone(),
            entry_ttl: TTL,
            attr_ttl: TTL,
        });

        for (path, &ino) in &*self.reverse_map.read().await {
            let path_cloned = path.clone();
            let file_info: PathInfo = self
                .afc
                .lock()
                .await
                .get_file_info(path_cloned.clone())
                .await
                .map_err(|e| {
                    eprintln!("get_file_info failed for {path_cloned}: {e}");
                    Into::<Errno>::into(libc::EIO)
                })?
                .into();

            let attr = file_info.to_rfuse_file_attr(ino);
            entries.push(DirectoryEntryPlus {
                inode: ino,
                generation: 0,
                attr,
                kind: FileType::RegularFile,
                name: path.clone().into(),
                offset: ino as _,
                entry_ttl: TTL,
                attr_ttl: TTL,
            });
        }

        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.offset > offset as i64)
            .map(Ok)
            .collect();

        Ok(ReplyDirectoryPlus {
            entries: futures_util::stream::iter(filtered),
        })
    }

    async fn access(&self, _req: Request, inode: u64, _mask: u32) -> FResult<()> {
        println!("access inode={}", inode);
        // We accept all access for now.
        Ok(())
    }

    async fn statfs(&self, _req: Request, _inode: u64) -> FResult<ReplyStatFs> {
        let device_info = self.afc.lock().await.get_device_info().await.unwrap();

        let totalspace = device_info.total_bytes;
        let blocksize = device_info.block_size;
        let freespace = device_info.free_bytes;

        Ok(ReplyStatFs {
            blocks: (totalspace / blocksize) as _,
            bfree: (freespace / blocksize) as _,
            bavail: (freespace / blocksize) as _,
            files: 1_000_000_000,
            ffree: 1_000_000_000,
            bsize: blocksize as _,
            namelen: 255,
            frsize: blocksize as _,
        })
    }
    async fn forget(&self, req: Request, inode: rfuse3::Inode, nlookup: u64) -> () {
        println!("forget inode={inode} nlookup={nlookup}");
    }
    async fn flush(
        &self,
        req: Request,
        inode: rfuse3::Inode,
        fh: u64,
        lock_owner: u64,
    ) -> FResult<()> {
        println!("flushing");
        Ok(())
    }

    async fn fsync(
        &self,
        req: Request,
        inode: rfuse3::Inode,
        fh: u64,
        datasync: bool,
    ) -> FResult<()> {
        println!("fsync");
        Ok(())
    }
    async fn getxattr(
        &self,
        _req: Request,
        inode: u64,
        name: &OsStr,
        _size: u32,
    ) -> FResult<ReplyXAttr> {
        println!(
            "Getting extended attributes: inode={}, name={:?}",
            inode, name
        );
        Err(libc::ENOTSUP.into())
    }

    async fn listxattr(&self, _req: Request, inode: u64, _size: u32) -> FResult<ReplyXAttr> {
        println!("Listing extended attributes: inode={}", inode);
        Ok(ReplyXAttr::Data(Vec::new().into()))
    }

    async fn open(&self, _req: Request, inode: u64, flags: u32) -> FResult<ReplyOpen> {
        println!("open inode={inode}");

        let path = self
            .inode_map
            .read()
            .await
            .get(&inode)
            .cloned()
            .ok_or(libc::ENOENT)?;

        let mode = match (flags as libc::c_int) & libc::O_ACCMODE {
            libc::O_RDONLY => AfcFopenMode::RdOnly,
            libc::O_WRONLY => AfcFopenMode::WrOnly,
            libc::O_RDWR => AfcFopenMode::Rw,
            _ => return Err(libc::EPERM.into()),
        };

        let mut afc = self.afc.lock().await;
        let fd_obj = afc.open(path, mode).await.map_err(|_| libc::EIO)?;
        let afc_fd = fd_obj.as_raw_fd();

        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        self.open_fds.lock().await.insert(fh, afc_fd);

        Ok(ReplyOpen { fh, flags: 0 })
    }

    async fn opendir(&self, _req: Request, inode: u64, _flags: u32) -> FResult<ReplyOpen> {
        println!("Opening directory: inode={}", inode);

        Ok(ReplyOpen { fh: 1, flags: 0 })
    }
}
