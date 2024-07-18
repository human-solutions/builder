use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::ValueWrapper;

use super::profile::Profile;

#[derive(Default, Serialize)]
pub struct Target {
    #[serde(rename = "triple")]
    pub name: Option<String>,
    pub profiles: Vec<Profile>,
}

impl Target {
    pub fn has_profile(&self, profile_name: &Option<String>) -> Option<usize> {
        self.profiles.iter().position(|p| p.name == *profile_name)
    }

    pub fn push(&mut self, profile: Option<String>, output: ValueWrapper) -> Result<()> {
        let profile = Profile::new(profile, output).context(format!(
            "target-add-profile:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.profiles.push(profile);

        Ok(())
    }

    pub fn new(
        name: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        let profile = Profile::new(profile, output).context(format!(
            "create-target:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            profiles: vec![profile],
        })
    }
}
