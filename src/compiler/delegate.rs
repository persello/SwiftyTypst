use crate::{AutocompleteResult, CompilationResult, HighlightResult};

pub trait TypstCompilerDelegate: Send + Sync {
    fn compilation_finished(&self, result: CompilationResult);
    fn highlighting_finished(&self, path: String, result: Vec<HighlightResult>);
    fn autocomplete_finished(&self, path: String, result: Vec<AutocompleteResult>);
}
