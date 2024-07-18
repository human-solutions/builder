use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::ValueWrapper;

use super::target::Target;

#[derive(Default, Serialize)]
pub struct Assembly {
    #[serde(rename = "assembly")]
    pub name: Option<String>,
    pub targets: Vec<Target>,
}

impl Assembly {
    pub fn has_target(&self, target_name: &Option<String>) -> Option<usize> {
        self.targets.iter().position(|t| t.name == *target_name)
    }

    pub fn push(
        &mut self,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<()> {
        let target = Target::new(target, profile, output).context(format!(
            "assembly-add-target:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.targets.push(target);

        Ok(())
    }

    pub fn new(
        name: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        let target = Target::new(target, profile, output).context(format!(
            "create-assembly:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            targets: vec![target],
        })
    }
}
