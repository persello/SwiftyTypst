use typst::eval::Tracer;

use super::TypstCompiler;

pub enum CompilationResult {
    Document { data: Vec<u8> },
    Errors { errors: Vec<String> },
}

impl TypstCompiler {
    pub fn compile(&self) -> CompilationResult {
        if let Ok(mut world) = self.world.write() {
            world.reset();

            let mut tracer = Tracer::new();

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
}
