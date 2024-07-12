use std::collections::HashMap;

use super::Binary;

pub enum BinaryState {
    Installed { version: String },
    NotInstalled,
}

pub struct BinariesStatus(HashMap<Binary, BinaryState>);

impl From<HashMap<Binary, BinaryState>> for BinariesStatus {
    fn from(value: HashMap<Binary, BinaryState>) -> Self {
        Self(value)
    }
}
