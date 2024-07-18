use serde::Serialize;

#[derive(Default, Serialize)]
pub enum Installer {
    Binstall(String),
    #[default]
    Cargo,
    Shell(String),
    Plugin(String),
}

impl Installer {
    fn install(&self) {
        todo!()
    }
}
