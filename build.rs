fn main() {
    uniffi::generate_scaffolding("src/typst_bindings.udl").unwrap();
}