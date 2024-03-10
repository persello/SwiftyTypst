use std::path::PathBuf;

use once_cell::unsync::OnceCell;
use typst::{
    diag::FileResult,
    foundations::Bytes,
    syntax::Source,
    syntax::{package::PackageSpec, FileId},
};

use super::file_manager::FileManager;

/// Holds canonical data for all paths pointing to the same entity.
pub struct PathSlot {
    id: FileId,
    path: PathBuf,
    package: Option<PackageSpec>,
    source: OnceCell<FileResult<Source>>,
    buffer: OnceCell<FileResult<Bytes>>,
}

impl PathSlot {
    pub fn new(id: FileId, path: PathBuf, package: &Option<&PackageSpec>) -> Self {
        Self {
            id,
            path,
            package: package.cloned(),
            source: OnceCell::new(),
            buffer: OnceCell::new(),
        }
    }

    fn package_string(&self) -> Option<String> {
        self.package.as_ref().map(|p| p.to_string())
    }

    #[allow(clippy::borrowed_box)]
    pub fn source(&self, reader: &Box<dyn FileManager>) -> FileResult<Source> {
        self.source
            .get_or_init(|| {
                let buf = reader.read(
                    self.path.to_str().unwrap().to_owned(),
                    self.package_string(),
                )?;

                let text = Self::decode_utf8(buf)?;
                Ok(Source::new(self.id, text))
            })
            .clone()
    }

    #[allow(clippy::borrowed_box)]
    pub fn file(&self, reader: &Box<dyn FileManager>) -> FileResult<Bytes> {
        self.buffer
            .get_or_init(|| {
                reader
                    .read(
                        self.path.to_str().unwrap().to_owned(),
                        self.package_string(),
                    )
                    .map(Bytes::from)
                    .map_err(Into::into)
            })
            .clone()
    }

    /// Decode UTF-8 with an optional BOM.
    fn decode_utf8(buf: Vec<u8>) -> FileResult<String> {
        Ok(if buf.starts_with(b"\xef\xbb\xbf") {
            // Remove UTF-8 BOM.
            std::str::from_utf8(&buf[3..])?.into()
        } else {
            // Assume UTF-8.
            String::from_utf8(buf)?
        })
    }
}
