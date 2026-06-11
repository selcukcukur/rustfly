use std::fmt;
use std::io;
use std::string::FromUtf8Error;
use std::time::SystemTime;

pub type Result<T> = std::result::Result<T, RustflyError>;

#[derive(Debug)]
pub enum RustflyError {
    Io(io::Error),
    InvalidUtf8(FromUtf8Error),
    InvalidPath(String),
    InvalidDriverName,
    DriverNotFound(String),
    DriverAlreadyRegistered(String),
    DefaultDriverMissing,
    RegistryPoisoned,
    LockPoisoned,
    Unsupported(&'static str),
}

impl fmt::Display for RustflyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::InvalidUtf8(error) => write!(formatter, "invalid utf-8 contents: {error}"),
            Self::InvalidPath(path) => write!(formatter, "invalid storage path: {path}"),
            Self::InvalidDriverName => write!(formatter, "driver name cannot be empty"),
            Self::DriverNotFound(name) => write!(formatter, "storage driver not found: {name}"),
            Self::DriverAlreadyRegistered(name) => {
                write!(formatter, "storage driver is already registered: {name}")
            }
            Self::DefaultDriverMissing => {
                write!(formatter, "default storage driver is not configured")
            }
            Self::RegistryPoisoned => write!(formatter, "storage registry is poisoned"),
            Self::LockPoisoned => write!(formatter, "storage lock is poisoned"),
            Self::Unsupported(operation) => {
                write!(formatter, "operation is unsupported: {operation}")
            }
        }
    }
}

impl std::error::Error for RustflyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidUtf8(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for RustflyError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FromUtf8Error> for RustflyError {
    fn from(value: FromUtf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Metadata {
    path: String,
    kind: EntryKind,
    len: u64,
    modified: Option<SystemTime>,
}

impl Metadata {
    pub fn new(
        path: impl Into<String>,
        kind: EntryKind,
        len: u64,
        modified: Option<SystemTime>,
    ) -> Self {
        Self {
            path: path.into(),
            kind,
            len,
            modified,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn is_file(&self) -> bool {
        self.kind == EntryKind::File
    }

    pub fn is_directory(&self) -> bool {
        self.kind == EntryKind::Directory
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }
}
