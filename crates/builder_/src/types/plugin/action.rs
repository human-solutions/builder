use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::ValueWrapper;

use super::assembly::Assembly;

#[derive(Serialize)]
pub struct Action {
    #[serde(rename = "action")]
    pub name: Option<String>,
    pub assemblies: Vec<Assembly>,
}

impl Action {
    pub fn has_assembly(&self, assembly_name: &Option<String>) -> Option<usize> {
        self.assemblies
            .iter()
            .position(|a| a.name == *assembly_name)
    }

    pub fn push(
        &mut self,
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<()> {
        let assembly = Assembly::new(assembly, target, profile, output).context(format!(
            "action-add-assembly:'{}'",
            self.name.as_ref().unwrap_or(&"".to_string())
        ))?;

        self.assemblies.push(assembly);

        Ok(())
    }

    pub fn new(
        name: Option<String>,
        assembly: Option<String>,
        target: Option<String>,
        profile: Option<String>,
        output: ValueWrapper,
    ) -> Result<Self> {
        let assembly = Assembly::new(assembly, target, profile, output).context(format!(
            "create_action:'{}'",
            name.as_ref().unwrap_or(&"".to_string())
        ))?;

        Ok(Self {
            name,
            assemblies: vec![assembly],
        })
    }
}
