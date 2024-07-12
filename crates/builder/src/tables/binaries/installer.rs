use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub enum Installer {
    Binstall(String),
    #[default]
    Cargo,
    Custom(String),
}

impl Installer {
    fn install(&self) {
        todo!()
    }
}
