use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::types::ValueWrapper;

#[derive(Default, Serialize)]
pub struct Profile {
    #[serde(rename = "profile")]
    pub name: Option<String>,
    pub output: Value,
}

impl Profile {
    pub fn new(name: Option<String>, output: ValueWrapper) -> Result<Self> {
        if let ValueWrapper::Single(output) = output {
            Ok(Self { name, output })
        } else {
            Err(anyhow::Error::msg(
                "Expected output data from table but output data defined as table array",
            ))
        }
    }
}
