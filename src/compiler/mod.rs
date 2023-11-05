pub mod autocomplete;
pub mod compile;
pub mod delegate;

use std::sync::{Arc, RwLock};

use typst::diag::FileError;

use crate::cli_glue::{file_manager::FileManager, SystemWorld};

#[derive(Clone)]
pub struct TypstCompiler {
    pub(crate) world: Arc<RwLock<SystemWorld>>,
}

impl TypstCompiler {
    pub fn new(file_manager: Box<dyn FileManager>, main: String) -> Self {
        Self {
            world: Arc::new(RwLock::new(SystemWorld::new(file_manager, main.into()))),
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
