use std::{
    error::Error,
    fmt::{Debug, Display},
    path::PathBuf,
};

use typst::diag::FileError;

pub enum FileManagerError {
    NotFound,
    AccessDenied,
    IsDirectory,
    NotSource,
    InvalidUtf8,
    FfiCallbackError,
    Other,
}

impl Error for FileManagerError {}

impl Display for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileManagerError::NotFound => write!(f, "file not found"),
            FileManagerError::AccessDenied => write!(f, "access denied"),
            FileManagerError::IsDirectory => write!(f, "is a directory"),
            FileManagerError::NotSource => write!(f, "not a source"),
            FileManagerError::InvalidUtf8 => write!(f, "invalid UTF-8"),
            FileManagerError::FfiCallbackError => write!(f, "FFI callback error"),
            FileManagerError::Other => write!(f, "other"),
        }
    }
}

impl Debug for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileManagerError::NotFound => write!(f, "file not found"),
            FileManagerError::AccessDenied => write!(f, "access denied"),
            FileManagerError::IsDirectory => write!(f, "is a directory"),
            FileManagerError::NotSource => write!(f, "not a source"),
            FileManagerError::InvalidUtf8 => write!(f, "invalid UTF-8"),
            FileManagerError::FfiCallbackError => write!(f, "FFI callback error"),
            FileManagerError::Other => write!(f, "other"),
        }
    }
}

impl From<FileManagerError> for FileError {
    fn from(val: FileManagerError) -> Self {
        match val {
            FileManagerError::NotFound => FileError::NotFound(PathBuf::from("")),
            FileManagerError::AccessDenied => FileError::AccessDenied,
            FileManagerError::IsDirectory => FileError::IsDirectory,
            FileManagerError::NotSource => FileError::NotSource,
            FileManagerError::InvalidUtf8 => FileError::InvalidUtf8,
            _ => FileError::Other(Some(val.to_string().into())),
        }
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for FileManagerError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        FileManagerError::FfiCallbackError
    }
}

pub trait FileManager: Send + Sync {
    fn read(&self, path: String) -> Result<Vec<u8>, FileManagerError>;
    fn write(&self, path: String, data: Vec<u8>) -> Result<(), FileManagerError>;
    fn exists(&self, path: String) -> Result<bool, FileManagerError>;
    fn create_directory(&self, path: String) -> Result<(), FileManagerError>;
}
