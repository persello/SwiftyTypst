use typst::{diag::SourceDiagnostic, eval::Tracer, syntax::Source, World};

use crate::SourceRange;

use super::TypstCompiler;

pub struct CompilationError {
    pub severity: typst::diag::Severity,
    pub source_path: Option<String>,
    pub range: Option<SourceRange>,
    pub message: String,
    pub hints: Vec<String>,
}

pub enum CompilationResult {
    Document {
        data: Vec<u8>,
        warnings: Vec<CompilationError>,
    },
    Errors {
        errors: Vec<CompilationError>,
    },
}

impl TypstCompiler {
    pub fn compile(&self) -> CompilationResult {
        if let Ok(world) = self.world.read() {
            // world.reset();

            let mut tracer = Tracer::new();

            let result = typst::compile(&(*world), &mut tracer);

            // Needed because otherwise we can't call self.diagnostic_to_error.
            drop(world);

            let final_result = match result {
                Ok(doc) => {
                    let pdf = typst::export::pdf(&doc, None, None);
                    let warnings = tracer.warnings();
                    CompilationResult::Document {
                        data: pdf,
                        warnings: warnings
                            .iter()
                            .map(|e| self.diagnostic_to_error(e.clone()))
                            .collect(),
                    }
                }
                Err(errors) => CompilationResult::Errors {
                    errors: errors
                        .iter()
                        .map(|e| self.diagnostic_to_error(e.clone()))
                        .collect(),
                },
            };

            return final_result;
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

        let range: Option<SourceRange> = if let Some(source) = source {
            if let Some(range) = source.range(span) {
                SourceRange::from_range(range, &source)
            } else {
                None
            }
        } else {
            None
        };

        CompilationError {
            severity: diagnostic.severity,
            source_path,
            range,
            message: diagnostic.message.to_string(),
            hints: diagnostic.hints.iter().map(Into::into).collect(),
        }
    }
}
