use std::{
    error::Error,
    fmt::{Debug, Display},
    path::PathBuf,
};

use typst::diag::FileError;

pub enum FileReaderError {
    NotFound,
    AccessDenied,
    IsDirectory,
    NotSource,
    InvalidUtf8,
    FfiCallbackError,
    Other,
}

impl Error for FileReaderError {}

impl Display for FileReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileReaderError::NotFound => write!(f, "file not found"),
            FileReaderError::AccessDenied => write!(f, "access denied"),
            FileReaderError::IsDirectory => write!(f, "is a directory"),
            FileReaderError::NotSource => write!(f, "not a source"),
            FileReaderError::InvalidUtf8 => write!(f, "invalid UTF-8"),
            FileReaderError::FfiCallbackError => write!(f, "FFI callback error"),
            FileReaderError::Other => write!(f, "other"),
        }
    }
}

impl Debug for FileReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileReaderError::NotFound => write!(f, "file not found"),
            FileReaderError::AccessDenied => write!(f, "access denied"),
            FileReaderError::IsDirectory => write!(f, "is a directory"),
            FileReaderError::NotSource => write!(f, "not a source"),
            FileReaderError::InvalidUtf8 => write!(f, "invalid UTF-8"),
            FileReaderError::FfiCallbackError => write!(f, "FFI callback error"),
            FileReaderError::Other => write!(f, "other"),
        }
    }
}

impl From<FileReaderError> for FileError {
    fn from(val: FileReaderError) -> Self {
        match val {
            FileReaderError::NotFound => FileError::NotFound(PathBuf::from("")),
            FileReaderError::AccessDenied => FileError::AccessDenied,
            FileReaderError::IsDirectory => FileError::IsDirectory,
            FileReaderError::NotSource => FileError::NotSource,
            FileReaderError::InvalidUtf8 => FileError::InvalidUtf8,
            _ => FileError::Other(Some(val.to_string().into())),
        }
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for FileReaderError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        FileReaderError::FfiCallbackError
    }
}

pub trait FileReader: Send + Sync {
    fn read(&self, path: String) -> Result<Vec<u8>, FileReaderError>;
}
