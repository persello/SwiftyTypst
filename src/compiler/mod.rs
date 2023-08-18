pub mod autocomplete;
pub mod compile;
pub mod highlight;

use std::sync::RwLock;

use typst::diag::FileError;

use crate::cli_glue::{file_reader::FileReader, SystemWorld};

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
}
