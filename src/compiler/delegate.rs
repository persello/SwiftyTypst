use crate::{AutocompleteResult, CompilationResult};

pub trait TypstCompilerDelegate: Send + Sync {
    fn compilation_finished(&self, result: CompilationResult);
}

pub trait TypstSourceDelegate: Send + Sync {
    fn autocomplete_finished(&self, result: Vec<AutocompleteResult>);
}
