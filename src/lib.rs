uniffi::include_scaffolding!("Typst");

mod cli_glue;
mod compiler;

pub use cli_glue::file_manager::{FileManager, FileManagerError};
pub use cli_glue::fonts::FontDefinition;
pub use compiler::{
    autocomplete::{AutocompleteKind, AutocompleteResult},
    compile::{CompilationError, CompilationResult},
    highlight::HighlightResult,
    TypstCompiler,
};

pub use typst::{
    diag::{FileError, Severity},
    ide::Tag,
};

macro_rules! st_log {
    ($($arg:tt)*) => {
        {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let fn_name = type_name_of(f);
            let fn_name = fn_name.strip_suffix("::f").unwrap();
            eprintln!("SwiftyTypst - {}: {}", fn_name, format!($($arg)*));
        }
    };
}

pub(crate) use st_log;
