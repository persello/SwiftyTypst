use std::{path::PathBuf, sync::RwLock};

use cli_glue::SystemWorld;
use typst::{
    diag::{FileError},
    World,
};

uniffi::include_scaffolding!("Typst");

mod cli_glue;

pub enum CompilationResult {
    Document { data: Vec<u8> },
    Errors { errors: Vec<String> },
}

pub struct TypstCompiler {
    world: RwLock<SystemWorld>,
}

impl TypstCompiler {
    pub fn new(root: String) -> Self {
        Self {
            world: RwLock::new(SystemWorld::new(PathBuf::from(root), &[])),
        }
    }

    pub fn set_main(&self, main: String) -> Result<(), FileError> {
        if let Ok(mut world) = self.world.write() {
            let main_buf = world.root.join(main);
            println!("Resolving main file \"{}\".", main_buf.display());
            world.main = world.resolve(&main_buf)?;
            println!("Main file found.");
            Ok(())
        } else {
            panic!("Failed to lock world.")
        }
    }

    pub fn compile(&self) -> CompilationResult {
        if let Ok(world) = self.world.read() {
            let result = typst::compile(&(*world));
            println!("Compilation result: {:?}", result);

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
