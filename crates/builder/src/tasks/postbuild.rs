use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    anyhow::{Context, Result},
    generate::Generator,
    tasks::{Config, Task},
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct PostbuildTasks(Vec<Task>);

impl PostbuildTasks {
    pub fn from_value(value: &Value) -> Result<Self> {
        let mut tasks = Vec::new();

        for (tool, tool_val) in value.as_object().context("Invalid builder metadata")? {
            for item in tool_val
                .as_array()
                .context(format!("Invalid tasks for tool '{tool}'"))?
            {
                tasks.push(Task::from_value(tool, item)?);
            }
        }

        Ok(Self(tasks))
    }

    pub fn run(&self, config: &Config) -> Result<()> {
        let mut generator = Generator::default();
        let mut watched = HashSet::new();

        for task in &self.0 {
            task.run(config, &mut generator, &mut watched)?;
        }

        generator.write(config)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
