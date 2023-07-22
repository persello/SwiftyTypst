use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    path::PathBuf,
};

use chrono::Datelike;
use comemo::Prehashed;
use once_cell::unsync::OnceCell;
use typst::{
    diag::{FileError, FileResult},
    eval::{Datetime, Library},
    file::FileId,
    font::{Font, FontBook},
    syntax::Source,
    util::{Bytes, PathExt},
    World,
};

use super::{
    file_reader::FileReader,
    font_searcher::{FontSearcher, FontSlot},
    path_hash::PathHash,
    path_slot::PathSlot,
};

/// A world that provides access to the operating system.
pub struct SystemWorld {
    pub root: PathBuf,
    pub main: FileId,
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    fonts: Vec<FontSlot>,
    hashes: RefCell<HashMap<FileId, FileResult<PathHash>>>,
    paths: RefCell<HashMap<PathHash, PathSlot>>,
    today: OnceCell<Option<Datetime>>,

    // Custom file reader for asking the main program to read files.
    file_reader: Box<dyn FileReader>,
}

unsafe impl Sync for SystemWorld {}

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
            main: FileId::detached(),
            today: OnceCell::new(),
            file_reader,
        }
    }
}

impl World for SystemWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> Source {
        self.source(self.main).unwrap()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id)?.source(&self.file_reader)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get(&self.file_reader)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id)?.file(&self.file_reader)
    }

    fn today(&self, offset: Option<i64>) -> Option<typst::eval::Datetime> {
        *self.today.get_or_init(|| {
            let naive = match offset {
                None => chrono::Local::now().naive_local(),
                Some(o) => (chrono::Utc::now() + chrono::Duration::hours(o)).naive_utc(),
            };
    
            typst::eval::Datetime::from_ymd(
                naive.year(),
                naive.month().try_into().ok()?,
                naive.day().try_into().ok()?,
            )
        })
    }
}

impl SystemWorld {
    fn slot(&self, id: FileId) -> FileResult<RefMut<PathSlot>> {
        let mut system_path = PathBuf::new();
        let hash = self
            .hashes
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                // Determine the root path relative to which the file path
                // will be resolved.
                let root = match id.package() {
                    Some(spec) => super::package::prepare_package(spec)?,
                    None => self.root.clone(),
                };

                // Join the path to the root. If it tries to escape, deny
                // access. Note: It can still escape via symlinks.
                system_path = root.join_rooted(id.path()).ok_or(FileError::AccessDenied)?;

                PathHash::new(&system_path)
            })
            .clone()?;

        Ok(RefMut::map(self.paths.borrow_mut(), |paths| {
            paths
                .entry(hash)
                .or_insert_with(|| PathSlot::new(id, system_path))
        }))
    }

    pub fn reset(&mut self) {
        self.hashes.borrow_mut().clear();
        self.paths.borrow_mut().clear();
    }

    pub fn set_main(&mut self, path: PathBuf) -> FileResult<()> {
        println!("main: {:?}", path);
        self.main = FileId::new(None, &self.root.join(path));
        Ok(())
    }
}
