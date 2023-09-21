use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    path::PathBuf,
};

use chrono::Datelike;
use comemo::Prehashed;
use once_cell::unsync::OnceCell;
use typst::{
    diag::FileResult,
    eval::{Bytes, Datetime, Library},
    font::{Font, FontBook},
    syntax::FileId,
    syntax::{Source, VirtualPath},
    World,
};

use super::{
    file_reader::FileReader, font_reader::FontReader, path_hash::PathHash, path_slot::PathSlot,
};

use crate::st_log;

/// A world that provides access to the operating system.
pub struct SystemWorld {
    pub main: FileId,
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    fonts: HashMap<usize, Font>,
    hashes: RefCell<HashMap<FileId, FileResult<PathHash>>>,
    paths: RefCell<HashMap<PathHash, PathSlot>>,
    today: OnceCell<Option<Datetime>>,

    // Custom file reader for asking the main program to read files.
    file_reader: Box<dyn FileReader>,
}

unsafe impl Sync for SystemWorld {}

impl SystemWorld {
    pub fn new(
        file_reader: Box<dyn FileReader>,
        font_reader: Box<dyn FontReader>,
        main: PathBuf,
    ) -> Self {
        st_log!("Initializing system world with main file: {:?}.", main);

        let vpath = VirtualPath::new(main);

        let font_definitions = font_reader.fonts();
        let mut book = FontBook::new();
        let mut font_map = HashMap::<usize, Font>::new();

        let fonts = font_definitions.into_iter().flatten();

        for (i, font) in fonts.enumerate() {
            book.push(font.info().clone());
            font_map.insert(i, font);
        }

        Self {
            library: Prehashed::new(typst_library::build()),
            book: Prehashed::new(book),
            fonts: font_map,
            hashes: RefCell::default(),
            paths: RefCell::default(),
            main: FileId::new(None, vpath),
            today: OnceCell::new(),
            file_reader,
        }
    }
}

impl World for SystemWorld {
    fn library(&self) -> &Prehashed<Library> {
        st_log!("Getting library.");
        &self.library
    }

    fn main(&self) -> Source {
        st_log!("Getting main source.");
        self.source(self.main).unwrap()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        st_log!("Getting source for file {:?}.", id);
        self.slot(id)?.source(&self.file_reader)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        st_log!("Getting font book.");
        &self.book
    }

    fn font(&self, id: usize) -> Option<Font> {
        st_log!("Getting font {}.", id);
        if let Some(font) = self.fonts.get(&id) {
            return Some(font.clone());
        } else {
            return None;
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        st_log!("Getting file {:?}.", id);
        self.slot(id)?.file(&self.file_reader)
    }

    fn today(&self, offset: Option<i64>) -> Option<typst::eval::Datetime> {
        st_log!("Getting today's date.");
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
        st_log!("Getting slot for file {:?}.", id);
        let mut system_path = PathBuf::new();
        let hash = self
            .hashes
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                st_log!("Hashing file {:?} in package {:?}.", id, id.package());

                if id.package().is_some() {
                    super::package::prepare_package(id.package().unwrap())?;
                }

                // Join the path to the root. If it tries to escape, deny
                // access. Note: It can still escape via symlinks.
                system_path = id.vpath().as_rooted_path().to_path_buf();

                st_log!("Resolved file {:?} to {:?} for hashing.", id, system_path);

                PathHash::new(&system_path)
            })
            .clone()?;

        st_log!("Hash is {:?}.", hash);

        Ok(RefMut::map(self.paths.borrow_mut(), |paths| {
            paths
                .entry(hash)
                .or_insert_with(|| PathSlot::new(id, system_path))
        }))
    }

    pub fn reset(&mut self) {
        st_log!("Resetting system world.");
        self.hashes.borrow_mut().clear();
        self.paths.borrow_mut().clear();
    }

    pub fn set_main(&mut self, path: PathBuf) -> FileResult<()> {
        st_log!("Setting main file to {:?}.", path);

        let vpath = VirtualPath::new(path);

        self.main = FileId::new(None, vpath);
        Ok(())
    }
}
