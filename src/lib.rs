uniffi::include_scaffolding!("typst_bindings");

pub fn add(left: u32, right: u32) -> u32 {
    left + right
}
