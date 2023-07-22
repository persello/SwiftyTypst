use std::path::PathBuf;

use once_cell::unsync::OnceCell;
use typst::{file::FileId, diag::FileResult, syntax::Source, util::Bytes};

use super::file_reader::FileReader;

/// Holds canonical data for all paths pointing to the same entity.
pub struct PathSlot {
    id: FileId,
    system_path: PathBuf,
    source: OnceCell<FileResult<Source>>,
    buffer: OnceCell<FileResult<Bytes>>,
}

impl PathSlot {
    pub fn new(id: FileId, system_path: PathBuf) -> Self {
        Self {
            id,
            system_path,
            source: OnceCell::new(),
            buffer: OnceCell::new(),
        }
    }
    
    #[allow(clippy::borrowed_box)]
    pub fn source(&self, reader: &Box<dyn FileReader>) -> FileResult<Source> {
        self.source
            .get_or_init(|| {
                let buf = reader.read(self.system_path.to_str().unwrap().to_owned())?;
                let text = Self::decode_utf8(buf)?;
                Ok(Source::new(self.id, text))
            })
            .clone()
    }

    #[allow(clippy::borrowed_box)]
    pub fn file(&self, reader: &Box<dyn FileReader>) -> FileResult<Bytes> {
        self.buffer
            .get_or_init(|| reader.read(self.system_path.to_str().unwrap().to_owned()).map(Bytes::from).map_err(Into::into))
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