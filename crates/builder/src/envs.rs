use std::{collections::HashMap, env};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Envs {
    cargo: HashMap<String, String>,
    rustc: HashMap<String, String>,
    other: HashMap<String, String>,
}

impl Envs {
    pub fn gather() -> Self {
        let mut envs = Envs::default();

        env::vars().for_each(|(key, val)| {
            if key.starts_with("CARGO_") {
                envs.cargo.insert(key, val);
            } else if key.starts_with("RUSTC_") {
                envs.rustc.insert(key, val);
            } else {
                envs.other.insert(key, val);
            }
        });

        envs
    }
}
