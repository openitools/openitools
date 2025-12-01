use futures_util::Stream;
use openitools_idevice::afc::{AfcClient, AfcFopenMode, FileDescriptor, PathInfo};
use rfuse3::{
    raw::{prelude::*, Filesystem},
    Errno, MountOptions, Result as FResult,
};
use std::ffi::{c_int, OsStr};
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{Mutex, RwLock};

use crate::device_management::afc::FuseCommand;

const ENOENT: c_int = 2;
const EIO: c_int = 5;

const TTL: Duration = Duration::from_secs(1); // 1 second

#[derive(Clone, Debug)]
pub struct File {
    path: String,
    fd: Arc<RwLock<Option<u64>>>,
}
pub struct AfcFS {
    inode_map: RwLock<HashMap<u64, File>>,
    reverse_map: RwLock<HashMap<String, u64>>,

    afc: Mutex<AfcClient>,

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
            inode_map.insert(
                ino,
                File {
                    path: path.clone(),
                    fd: RwLock::new(None).into(),
                },
            );
            reverse_map.insert(path, ino);
        }

        Self {
            inode_map: RwLock::new(inode_map),
            reverse_map: RwLock::new(reverse_map),
            afc: Mutex::new(afc),
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
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 4096,
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

        let file = self.inode_map.write().await.remove(&inode).unwrap();
        self.reverse_map.write().await.remove(&file.path);

        if self.inode_map.read().await.len() == 0 {
            self.shutdown_tx.send(());
        };

        Ok(())
    }

    async fn destroy(&self, _req: Request) {
        println!("Filesystem destruction");
    }

    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> FResult<ReplyEntry> {
        let name_str = name.to_str().ok_or_else(|| Into::<Errno>::into(ENOENT))?;
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
            return Err(ENOENT.into());
        }

        if let Some(&ino) = self.reverse_map.read().await.get(name_str) {
            let inode_map = self.inode_map.read().await;
            let file = inode_map
                .get(&ino)
                .ok_or_else(|| Into::<Errno>::into(ENOENT))?;

            println!("getting file info for {file:#?}");

            let path = &file.path;
            let file_info: PathInfo = self
                .afc
                .lock()
                .await
                .get_file_info(path)
                .await
                .map_err(|e| {
                    eprintln!("get_file_info failed: {e}");
                    EIO
                })?
                .into();

            println!("got info: {file_info:#?}");

            return Ok(ReplyEntry {
                ttl: TTL,
                attr: file_info.to_rfuse_file_attr(ino),
                generation: 0,
            });
        }

        Err(ENOENT.into())
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

        if let Some(file) = self.inode_map.read().await.get(&inode) {
            let File { path, .. } = &*file;
            println!("getting attribute of {path}");
            let file_info: PathInfo = self
                .afc
                .lock()
                .await
                .get_file_info(path)
                .await
                .map_err(|e| {
                    eprintln!("get_file_info failed: {e}");
                    EIO
                })?
                .into();
            println!("info: {file_info:#?}");

            return Ok(ReplyAttr {
                ttl: TTL,
                attr: file_info.to_rfuse_file_attr(inode),
            });
        }

        Err(ENOENT.into())
    }

    async fn read(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> FResult<ReplyData> {
        let file = match self.inode_map.read().await.get(&inode).cloned() {
            Some(p) => p,
            None => return Err(ENOENT.into()),
        };

        let mut afc_lock = self.afc.lock().await;
        println!("reading inode: {inode} path={}", &file.path);

        // TODO: buffered?
        let mut remote_file = tokio::io::BufReader::new(match file {
            File { path, fd } if fd.read().await.is_some() => unsafe {
                let fd = fd.read().await.unwrap_unchecked();
                println!("opening a file from fd: {fd}");
                FileDescriptor::new(&mut afc_lock, fd, path)
            },
            File { path, fd } => {
                println!("opening a new file");
                let remote_file = afc_lock
                    .open(path, AfcFopenMode::RdOnly)
                    .await
                    .map_err(|e| {
                        eprintln!("afc open failed: {e}");
                        EIO
                    })?;

                println!("created a file: {remote_file:#?}");

                *fd.write().await = Some(remote_file.as_raw_fd());

                remote_file
            }
        });
        let mut buf = vec![0u8; size as usize];

        if let Err(e) = remote_file.seek(std::io::SeekFrom::Start(offset)).await {
            eprintln!("seek failed: {e}");
        }

        let n = match remote_file.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("read failed: {e}");
                return Err(EIO.into());
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
            return Err(ENOENT.into());
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
            return Err(ENOENT.into());
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
                    Into::<Errno>::into(EIO)
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
        println!("statfs");
        Ok(ReplyStatFs {
            blocks: 1000,
            bfree: 800,
            bavail: 800,
            files: 100,
            ffree: 50,
            bsize: 4096,
            namelen: 255,
            frsize: 4096,
        })
    }

    async fn open(&self, _req: Request, inode: u64, _flags: u32) -> FResult<ReplyOpen> {
        println!("open inode={}", inode);
        Ok(ReplyOpen { fh: 2, flags: 0 })
    }
}
