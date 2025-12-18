#[cfg(target_os = "macos")]
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use chrono::TimeZone as _;
use idevice::afc::FileInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FSTree {
    pub path: String,
    pub info: PathInfo,
    pub children: Vec<FSTree>,
}

impl FSTree {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    pub fn add_child(&mut self, child: FSTree) {
        self.children.push(child);
    }
}

impl From<FileInfo> for PathInfo {
    fn from(value: FileInfo) -> Self {
        Self {
            size: value.size,
            blocks: value.blocks,
            nlink: value.st_nlink,
            creation: value.creation,
            modified: value.modified,
            file_type: FileType::from(value.st_ifmt),
        }
    }
}

impl From<rfuse3::FileType> for FileType {
    fn from(value: rfuse3::FileType) -> Self {
        use rfuse3::FileType as FT;
        match value {
            FT::NamedPipe => Self::NamedPipe,
            FT::CharDevice => Self::CharDevice,
            FT::BlockDevice => Self::BlockDevice,
            FT::Directory => Self::Directory,
            FT::RegularFile => Self::File,
            FT::Symlink => Self::Symlink,
            FT::Socket => Self::Socket,
        }
    }
}

impl From<FileType> for rfuse3::FileType {
    fn from(value: FileType) -> Self {
        use FileType as FT;
        match value {
            FT::NamedPipe => Self::NamedPipe,
            FT::CharDevice => Self::CharDevice,
            FT::BlockDevice => Self::BlockDevice,
            FT::Directory => Self::Directory,
            FT::File => Self::RegularFile,
            FT::Symlink => Self::Symlink,
            FT::Socket => Self::Socket,
            _ => unreachable!(),
        }
    }
}

fn naive_to_systemtime(ndt: chrono::NaiveDateTime) -> std::time::SystemTime {
    //  interpret the naive datetime as UTC
    let dt_utc = chrono::Utc.from_utc_datetime(&ndt);

    //  extract whole seconds and nanos
    let ts = dt_utc.timestamp(); // i64 seconds
    let nanos = dt_utc.timestamp_subsec_nanos();

    //  build SystemTime from UNIX_EPOCH + offset
    UNIX_EPOCH
        + std::time::Duration::from_secs(ts as u64)
        + std::time::Duration::from_nanos(nanos as u64)
}

impl PathInfo {
    pub fn to_rfuse_file_attr(self, ino: u64) -> rfuse3::raw::prelude::FileAttr {
        rfuse3::raw::prelude::FileAttr {
            ino,
            size: self.size as _,
            blocks: self.blocks as _,
            mtime: naive_to_systemtime(self.modified).into(),
            ctime: naive_to_systemtime(self.creation).into(),
            kind: self.file_type.into(),
            perm: 0o755,
            // default
            atime: SystemTime::now().into(),

            #[cfg(target_os = "macos")]
            crtime: SystemTime::now().into(),

            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,

            #[cfg(target_os = "macos")]
            flags: 0,
            blksize: 512,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PathInfo {
    pub size: usize,
    pub blocks: usize,
    pub nlink: String,
    pub creation: chrono::NaiveDateTime,
    pub modified: chrono::NaiveDateTime,
    pub file_type: FileType,
}

impl PathInfo {
    pub fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    pub fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum FileType {
    Directory,
    File,
    Symlink,

    // mostly for jailbroken devices, still not implemented yet
    // TODO: implement afc client for jailbroken devices
    CharDevice,
    BlockDevice,
    NamedPipe,
    Socket,

    #[default]
    Unknown,
}

impl From<String> for FileType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "S_IFDIR" => Self::Directory,
            "S_IFREG" => Self::File,
            "S_IFLNK" => Self::Symlink,

            "S_IFCHR" => Self::CharDevice,
            "S_IFBLK" => Self::BlockDevice,
            "S_IFIFO" => Self::NamedPipe,
            "S_IFSOCK" => Self::Socket,

            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Directory => write!(f, "directory"),
            Self::Symlink => write!(f, "symlink"),

            Self::CharDevice => write!(f, "character device"),
            Self::BlockDevice => write!(f, "block device"),
            Self::NamedPipe => write!(f, "named pipe(fifo)"),
            Self::Socket => write!(f, "socket"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
