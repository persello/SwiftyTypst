use std::{ops::Range, path::PathBuf, sync::RwLock};

use cli_glue::{file_reader::FileReader, SystemWorld};
use typst::{diag::FileError, file::FileId, ide::Tag, syntax::LinkedNode, util::PathExt, World};

uniffi::include_scaffolding!("Typst");

mod cli_glue;

pub use cli_glue::file_reader::FileReaderError;

macro_rules! st_log {
    ($($arg:tt)*) => {
        {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let fn_name = type_name_of(f);
            let fn_name = fn_name.strip_suffix("::f").unwrap();
            eprintln!("SwiftyTypst - {}: {}", fn_name, format!($($arg)*));
        }
    };
}

pub(crate) use st_log;

pub enum CompilationResult {
    Document { data: Vec<u8> },
    Errors { errors: Vec<String> },
}

pub struct HighlightResult {
    pub start: u64,
    pub end: u64,
    pub tag: String,
}

pub struct TypstCompiler {
    world: RwLock<SystemWorld>,
}

impl TypstCompiler {
    pub fn new(file_reader: Box<dyn FileReader>, main: String) -> Self {
        Self {
            world: RwLock::new(SystemWorld::new(file_reader, main.into())),
        }
    }

    pub fn set_main(&self, main: String) -> Result<(), FileError> {
        if let Ok(mut world) = self.world.write() {
            world.set_main(main.into())?;
            Ok(())
        } else {
            panic!("Failed to lock world.")
        }
    }

    pub fn notify_change(&self) {
        self.world.write().unwrap().reset();
    }

    pub fn compile(&self) -> CompilationResult {
        if let Ok(mut world) = self.world.write() {
            world.reset();

            let result = typst::compile(&(*world));

            if let Ok(doc) = result {
                let pdf = typst::export::pdf(&doc);
                CompilationResult::Document { data: pdf }
            } else {
                CompilationResult::Errors { errors: vec![] }
            }
        } else {
            panic!("Failed to lock world.")
        }
    }

    pub fn highlight(&self, file_path: String) -> Vec<HighlightResult> {
        let path = PathBuf::from(file_path);
        let Some(real_path) = self.world.read().unwrap().root.join_rooted(&path) else {
            return vec![];
        };

        let id = FileId::new(None, &real_path);
        let source = self.world.read().unwrap().source(id).unwrap();

        let node = LinkedNode::new(source.root());

        self.highlight_tree(&node)
            .iter()
            .map(|r| HighlightResult {
                start: r.0.start as u64,
                end: r.0.end as u64,
                tag: r.1.tm_scope().to_string(),
            })
            .collect()
    }

    fn highlight_tree(&self, node: &LinkedNode) -> Vec<(Range<usize>, Tag)> {
        let mut tags = vec![];

        if let Some(tag) = typst::ide::highlight(node) {
            tags.push((node.range(), tag));
        }

        for child in node.children() {
            tags.append(&mut self.highlight_tree(&child));
        }

        tags
    }
}
