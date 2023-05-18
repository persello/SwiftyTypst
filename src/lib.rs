use std::path::PathBuf;

uniffi::include_scaffolding!("Typst");

pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

struct SerifianWorld {
}

impl typst::World for SerifianWorld {
    fn library(&self) -> &Prehashed<typst::eval::Library> {
        todo!()
    }

    fn main(&self) -> &typst::syntax::Source {
        todo!()
    }

    fn resolve(&self, path: &std::path::Path) -> typst::diag::FileResult<typst::syntax::SourceId> {
        todo!()
    }

    fn source(&self, id: typst::syntax::SourceId) -> &typst::syntax::Source {
        todo!()
    }

    fn book(&self) -> &Prehashed<typst::font::FontBook> {
        todo!()
    }

    fn font(&self, id: usize) -> Option<typst::font::Font> {
        todo!()
    }

    fn file(&self, path: &std::path::Path) -> typst::diag::FileResult<typst::util::Buffer> {
        todo!()
    }
}

pub fn compile() -> Result<Vec<u8>, ()> {
    let input: PathBuf = PathBuf::from("test.typst");

    let world = SerifianWorld {};

    let result = typst::compile(&world).map_err(|_| ())?;

    let pdf = typst::export::pdf(&result);

    Ok(pdf)
}
