use typst::{diag::SourceDiagnostic, eval::Tracer, syntax::Source, World};

use super::TypstCompiler;

pub struct CompilationError {
    pub severity: typst::diag::Severity,
    pub source_path: Option<String>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub message: String,
    pub hints: Vec<String>,
}

pub enum CompilationResult {
    Document { data: Vec<u8> },
    Errors { errors: Vec<CompilationError> },
}

impl TypstCompiler {
    pub fn compile(&self) -> CompilationResult {
        if let Ok(mut world) = self.world.write() {
            world.reset();

            let mut tracer = Tracer::new();

            let result = typst::compile(&(*world), &mut tracer);

            // Needed because otherwise we can't call self.diagnostic_to_error.
            drop(world);

            match result {
                Ok(doc) => {
                    let pdf = typst::export::pdf(&doc);
                    CompilationResult::Document { data: pdf }
                }
                Err(errors) => CompilationResult::Errors {
                    errors: errors
                        .iter()
                        .map(|e| self.diagnostic_to_error(e.clone()))
                        .collect(),
                },
            }
        } else {
            panic!("Failed to lock world.")
        }
    }

    pub fn diagnostic_to_error(&self, diagnostic: SourceDiagnostic) -> CompilationError {
        let span = diagnostic.span;
        let (source, source_path): (Option<Source>, Option<String>) = if let Some(id) = span.id() {
            let vpath = id.vpath();
            let source = self.world.read().unwrap().source(id).unwrap();
            let path_string = vpath.as_rooted_path().to_string_lossy().to_string();
            (Some(source), Some(path_string))
        } else {
            (None, None)
        };

        let range = if let Some(source) = source {
            if let Some(range) = source.range(span) {
                Some((range.start, range.end))
            } else {
                None
            }
        } else {
            None
        };

        CompilationError {
            severity: diagnostic.severity,
            source_path,
            start: range.map(|r| r.0 as u64),
            end: range.map(|r| r.1 as u64),
            message: diagnostic.message.to_string(),
            hints: diagnostic.hints.iter().map(Into::into).collect(),
        }
    }
}
