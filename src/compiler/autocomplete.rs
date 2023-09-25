use std::path::PathBuf;

use typst::{
    ide::{Completion, CompletionKind},
    syntax::{FileId, VirtualPath},
    World,
};

use super::TypstCompiler;

pub enum AutocompleteKind {
    Syntax,
    Func,
    Param,
    Constant,
    Symbol,
    Type,
}

impl From<CompletionKind> for AutocompleteKind {
    fn from(value: CompletionKind) -> Self {
        match value {
            CompletionKind::Syntax => Self::Syntax,
            CompletionKind::Func => Self::Func,
            CompletionKind::Param => Self::Param,
            CompletionKind::Constant => Self::Constant,
            CompletionKind::Symbol(_) => Self::Symbol,
            CompletionKind::Type => Self::Type,
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
            completion: value.apply.unwrap_or(value.label.clone()).to_string(),
            label: value.label.to_string(),
            description: value.detail.unwrap_or_default().to_string(),
            kind: value.kind.into(),
        }
    }
}

impl TypstCompiler {
    pub fn autocomplete(
        &self,
        file_path: String,
        line: u64,
        column: u64,
    ) -> Vec<AutocompleteResult> {
        let path = PathBuf::from(file_path);
        let Ok(mut world) = self.world.write() else {
            return vec![];
        };

        let vpath = VirtualPath::new(path);

        world.reset();

        let id = FileId::new(None, vpath);
        let Ok(source) = world.source(id) else {
            return vec![];
        };

        let Some(position) = source
            .line_column_to_byte(line as usize, column as usize) else {
                return vec![];
            };

        let result = typst::ide::autocomplete(&(*world), &[], &source, position, false);

        let Some(completions) = result else {
            return vec![];
        };

        completions.1.into_iter().map(Into::into).collect()
    }
}
