use std::path::PathBuf;

uniffi::include_scaffolding!("Typst");

mod system_world;

pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

pub fn compile(path: String) -> Vec<u8> {
    let world = system_world::SystemWorld::new(PathBuf::from(path), &[]);

    let result = typst::compile(&world);

    if let Ok(doc) = result {
        let pdf = typst::export::pdf(&doc);
        pdf
    } else {
        Vec::new()
    }
}

