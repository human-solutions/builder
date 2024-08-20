uniffi::include_scaffolding!("library");

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}