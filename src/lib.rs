use std::path::{PathBuf, Path};

use typst::World;

uniffi::include_scaffolding!("Typst");

mod system_world;

pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

pub fn compile(root: String, main: String) -> Option<Vec<u8>> {
    let root_buf = PathBuf::from(root.clone());
    let mut world = system_world::SystemWorld::new(root_buf.clone(), &[]);
    println!("World created. Root path is \"{}\".", root);

    let main_buf = root_buf.join(main.clone());
    println!("Resolving main file \"{}\".", main_buf.display());
    world.main = world.resolve(&main_buf).ok()?;
    println!("Main file is \"{}\".", main);

    let result = typst::compile(&world);
    println!("Compilation result: {:?}", result);

    if let Ok(doc) = result {
        let pdf = typst::export::pdf(&doc);
        Some(pdf)
    } else {
        None
    }
}

