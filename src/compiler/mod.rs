pub mod autocomplete;
pub mod compile;
pub mod delegate;
pub mod highlight;

use std::sync::{Arc, Mutex, RwLock};

use typst::diag::FileError;

use crate::cli_glue::{file_manager::FileManager, SystemWorld};

use self::delegate::TypstCompilerDelegate;

#[derive(Clone)]
pub struct TypstCompiler {
    pub(crate) delegate: Arc<Mutex<Box<dyn TypstCompilerDelegate>>>,
    pub(crate) world: Arc<RwLock<SystemWorld>>,
}

impl TypstCompiler {
    pub fn new(
        delegate: Box<dyn TypstCompilerDelegate>,
        file_manager: Box<dyn FileManager>,
        main: String,
    ) -> Self {
        Self {
            delegate: Arc::new(Mutex::new(delegate)),
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
