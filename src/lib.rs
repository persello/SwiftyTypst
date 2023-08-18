use std::{ops::Range, path::PathBuf, sync::RwLock};

use cli_glue::{file_reader::FileReader, SystemWorld};
use typst::{
    diag::FileError,
    eval::Tracer,
    ide::{Completion, CompletionKind, Tag},
    syntax::{FileId, LinkedNode},
    util::PathExt,
    World,
};

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
    pub tag: Tag,
}

pub enum AutocompleteKind {
    Syntax,
    Func,
    Param,
    Constant,
    Symbol,
}

impl From<CompletionKind> for AutocompleteKind {
    fn from(value: CompletionKind) -> Self {
        match value {
            CompletionKind::Syntax => Self::Syntax,
            CompletionKind::Func => Self::Func,
            CompletionKind::Param => Self::Param,
            CompletionKind::Constant => Self::Constant,
            CompletionKind::Symbol(_) => Self::Symbol,
        }
    }
}

pub struct AutocompleteResult {
    pub kind: AutocompleteKind,
    pub label: String,
    pub completion: String,
    pub description: String,
}

impl From<Completion> for AutocompleteResult {
    fn from(value: Completion) -> Self {
        Self {
            completion: value.apply.unwrap_or_default().to_string(),
            label: value.label.to_string(),
            description: value.detail.unwrap_or_default().to_string(),
            kind: value.kind.into(),
        }
    }
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

            let mut tracer = Tracer::new(None);

            let result = typst::compile(&(*world), &mut tracer);

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

    pub fn autocomplete(&self, file_path: String, position: u64) -> Vec<AutocompleteResult> {
        let path = PathBuf::from(file_path);
        let Ok(mut world) = self.world.write() else {
            return vec![];
        };

        let Some(real_path) = world.root.join_rooted(&path) else {
            return vec![];
        };

        world.reset();

        let id = FileId::new(None, &real_path);
        let source = world.source(id).unwrap();

        let result = typst::ide::autocomplete(&(*world), &[], &source, position as usize, false);

        let Some(completions) = result else {
            return vec![];
        };

        completions.1.into_iter().map(Into::into).collect()
    }

    pub fn highlight(&self, file_path: String) -> Vec<HighlightResult> {
        let path = PathBuf::from(file_path);
        let Some(real_path) = self.world.read().unwrap().root.join_rooted(&path) else {
            return vec![];
        };

        self.world.write().unwrap().reset();

        let id = FileId::new(None, &real_path);
        let source = self.world.read().unwrap().source(id).unwrap();

        let node = LinkedNode::new(source.root());

        self.highlight_tree(&node)
            .iter()
            .map(|r| HighlightResult {
                start: r.0.start as u64,
                end: r.0.end as u64,
                tag: r.1,
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
