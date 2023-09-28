use crate::{AutocompleteResult, CompilationResult, HighlightResult};

pub trait TypstCompilerDelegate: Send + Sync {
    fn compilation_finished(&self, result: CompilationResult);
}

pub trait TypstSourceDelegate: Send + Sync {
    fn highlighting_finished(&self, result: Vec<HighlightResult>);
    fn autocomplete_finished(&self, result: Vec<AutocompleteResult>);
}
