use std::fs;

use anyhow::{Context, Result};
use cargo_metadata::camino::Utf8PathBuf;
use serde_json::Value;

pub struct Profiles(Vec<String>);

impl Profiles {
    pub fn gather(dir: &Utf8PathBuf) -> Result<Self> {
        let cargo_str = fs::read_to_string(dir.join("Cargo.toml"))?;

        let cargo_data: Value = toml::from_str(&cargo_str)?;

        let mut profiles = Vec::new();

        for (key, val) in cargo_data
            .as_object()
            .context("Failed to retrieve cargo data as object")?
        {
            if key == "profile" {
                for (profile, _) in val
                    .as_object()
                    .context("Failed to retrieve profile data as object")?
                {
                    profiles.push(profile.to_owned());
                }
            }
        }

        Ok(Self(profiles))
    }
    pub fn contains(&self, profile: &str) -> bool {
        self.0.iter().any(|p| p == profile)
    }
}
