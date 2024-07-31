use std::collections::HashSet;

use serde_json::Value;

use crate::{anyhow::Result, Config};

use crate::generate::Generator;

use super::{fontforge::FontForge, Assembly};

#[derive(Debug)]
pub struct PrebuildConfig {
    pub assemblies: Vec<Assembly>,
    pub fontforge: Option<FontForge>,
}

impl PrebuildConfig {
    pub fn from_json(json: &Value) -> Result<Self> {
        let mut assemblies = Vec::new();
        let mut fontforge = None;
        for (assembly_name, assembly_val) in json.as_object().unwrap() {
            if assembly_name == "fontforge" {
                fontforge = Some(serde_json::from_value(assembly_val.clone())?);
            } else if let Some(assembly_obj) = assembly_val.as_object() {
                for (key, val) in assembly_obj.iter() {
                    let mut assembly: Assembly = serde_json::from_value(val.clone()).unwrap();
                    assembly.name = if assembly_name == "*" {
                        None
                    } else {
                        Some(assembly_name.to_owned())
                    };
                    assembly.profile.clone_from(key);
                    assemblies.push(assembly);
                }
            } else {
                panic!("invalid assembly")
            }
        }
        Ok(Self {
            assemblies,
            fontforge,
        })
    }
}

impl PrebuildConfig {
    pub fn process(&self, conf: &Config) -> Result<()> {
        let mut watched = HashSet::new();
        watched.insert("Cargo.toml".to_string());
        watched.insert("src".to_string());

        let mut assembly_names = HashSet::new();

        if let Some(ff) = self.fontforge.as_ref() {
            ff.process(conf)?;
            watched.insert(ff.file.to_string());
        }
        let mut generator = Generator::default();

        log::trace!("Processing {:#?}", self.assemblies);

        // go through all named assemblies
        for assembly in &self.assemblies {
            let Some(name) = assembly.name.as_ref() else {
                continue;
            };
            assembly_names.insert(name.to_string());
            if conf.args.profile == assembly.profile {
                let change = assembly.process(conf, &mut generator, name, true)?;
                watched.extend(change.into_iter());
            }
        }

        // go through wildcard assemblies
        for assembly in self.assemblies.iter().filter(|a| a.name.is_none()) {
            for name in &assembly_names {
                if conf.args.profile == assembly.profile {
                    let change = assembly.process(conf, &mut generator, name, false)?;
                    watched.extend(change.into_iter());
                }
            }
        }
        generator.write(conf)?;
        for change in watched {
            println!("cargo::rerun-if-changed={}", change);
        }
        Ok(())
    }
}
