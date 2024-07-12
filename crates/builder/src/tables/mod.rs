mod binaries;
mod configs;
mod install;

use std::{fmt, str::FromStr};

pub use binaries::Binaries;
pub use configs::{Configs, Phase};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use serde_json::Value;

pub struct Tables {
    pub configs: Configs,
    pub binaries: Binaries,
}

impl<'a> Deserialize<'a> for Tables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_map(TableVisitor)
    }
}

struct TableVisitor;

impl<'a> Visitor<'a> for TableVisitor {
    type Value = Tables;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a configuration with nested keys from a toml file")
    }

    fn visit_map<M>(self, mut map: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'a>,
    {
        let mut configs = Configs::default();
        let mut binaries = Binaries::default();

        while let Some((key, val)) = map.next_entry::<String, Value>()? {
            match key.as_str() {
                "install" => binaries
                    .insert_batch_obj(&val)
                    .map_err(serde::de::Error::custom)?,
                phase if Phase::from_str(phase).is_ok() => {
                    configs
                        .insert(key.as_str(), &val)
                        .map_err(serde::de::Error::custom)?;
                }
                _ => continue,
            }
        }

        Ok(Tables { configs, binaries })
    }
}
