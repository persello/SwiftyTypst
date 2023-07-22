use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::Datelike;
use comemo::Prehashed;
use elsa::FrozenVec;
use once_cell::unsync::OnceCell;
use typst::{
    diag::FileResult,
    eval::Library,
    font::{Font, FontBook},
    syntax::{Source, SourceId},
    util::{Buffer, PathExt},
    World,
};

use super::{
    file_reader::FileReader,
    font_searcher::{FontSearcher, FontSlot},
    path_hash::PathHash,
};

/// A world that provides access to the operating system.
pub struct SystemWorld {
    pub root: PathBuf,
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    fonts: Vec<FontSlot>,
    hashes: RefCell<HashMap<PathBuf, FileResult<PathHash>>>,
    paths: RefCell<HashMap<PathHash, PathSlot>>,
    sources: FrozenVec<Box<Source>>,
    pub main: SourceId,
    file_reader: Box<dyn FileReader>,
}

unsafe impl Sync for SystemWorld {}

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Default)]
struct PathSlot {
    source: OnceCell<FileResult<SourceId>>,
    buffer: OnceCell<FileResult<Buffer>>,
}

impl SystemWorld {
    pub fn new(file_reader: Box<dyn FileReader>) -> Self {
        let mut searcher = FontSearcher::new();
        searcher.search(&[]);

        Self {
            root: PathBuf::from("."),
            library: Prehashed::new(typst_library::build()),
            book: Prehashed::new(searcher.book),
            fonts: searcher.fonts,
            hashes: RefCell::default(),
            paths: RefCell::default(),
            sources: FrozenVec::new(),
            main: SourceId::detached(),
            file_reader,
        }
    }
}

impl World for SystemWorld {
    fn root(&self) -> &Path {
        &self.root
    }

    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> &Source {
        self.source(self.main)
    }

    fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.slot(path)?
            .source
            .get_or_init(|| {
                let buf = self.file_reader.read(path.to_str().unwrap().to_owned())?;
                let text = String::from_utf8(buf)?;
                Ok(self.insert(path, text))
            })
            .clone()
    }

    fn source(&self, id: SourceId) -> &Source {
        &self.sources[id.as_u16() as usize]
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, id: usize) -> Option<Font> {
        let slot = &self.fonts[id];
        slot.font
            .get_or_init(|| {
                let data = self.file(&slot.path).ok()?;
                Font::new(data, slot.index)
            })
            .clone()
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        self.slot(path)?
            .buffer
            .get_or_init(|| {
                self.file_reader
                    .read(path.to_str().unwrap().to_owned())
                    .map(Buffer::from)
                    .map_err(Into::into)
            })
            .clone()
    }

    fn today(&self,offset:Option<i64>) -> Option<typst::eval::Datetime> {
        let naive = match offset {
            None => chrono::Local::now().naive_local(),
            Some(o) => (chrono::Utc::now() + chrono::Duration::hours(o)).naive_utc(),
        };

        typst::eval::Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }
}

impl SystemWorld {
    fn slot(&self, path: &Path) -> FileResult<RefMut<PathSlot>> {
        let mut hashes = self.hashes.borrow_mut();
        let hash = match hashes.get(path).cloned() {
            Some(hash) => hash,
            None => {
                let hash = PathHash::new(path);
                if let Ok(canon) = path.canonicalize() {
                    hashes.insert(canon.normalize(), hash.clone());
                }
                hashes.insert(path.into(), hash.clone());
                hash
            }
        }?;

        Ok(std::cell::RefMut::map(self.paths.borrow_mut(), |paths| {
            paths.entry(hash).or_default()
        }))
    }

    fn insert(&self, path: &Path, text: String) -> SourceId {
        let id = SourceId::from_u16(self.sources.len() as u16);
        let source = Source::new(id, path, text);
        self.sources.push(Box::new(source));
        id
    }

    pub fn reset(&mut self) {
        self.sources.as_mut().clear();
        self.hashes.borrow_mut().clear();
        self.paths.borrow_mut().clear();
    }
}
