use std::collections::HashSet;

use fs_err as fs;
use toml_edit::DocumentMut;

use crate::anyhow::{Context, Result};

use crate::generate::Generator;

use super::{fontforge::FontForge, Assembly, PrebuildArgs};

#[derive(Debug)]
pub struct PrebuildManifest {
    pub assemblies: Vec<Assembly>,
    pub fontforge: Option<FontForge>,
}

impl PrebuildManifest {
    pub fn try_parse(info: &PrebuildArgs) -> Result<Self> {
        Self::_try_parse(info).with_context(|| {
            format!(
                "Failed to parse prebuild manifest at: {}",
                info.manifest_dir
            )
        })
    }

    pub fn _try_parse(info: &PrebuildArgs) -> Result<Self> {
        let manifest_str = fs::read_to_string(info.manifest_dir.join("Cargo.toml"))?;
        let manifest = manifest_str.parse::<DocumentMut>()?;
        let val = &manifest
            .get("package")
            .context("Could not find package section in manifest")?
            .get("metadata")
            .context("Could not find package.metadata section in manifest")?
            .get("prebuild")
            .context("Could not find package.metadata.prebuild section in manifest")?;

        let names = val.as_table().context(
            "Could not find assembly name. Expected package.metadata.prebuild.<assembly>",
        )?;

        let mut assemblies = Vec::new();
        let mut fontforge = None;
        for (name, value) in names {
            if name == "fontforge" {
                fontforge = Some(FontForge::try_parse(value)?);
                continue;
            }

            for (profile, toml) in value.as_table().unwrap() {
                let ass = Assembly::try_parse(name, profile, toml)?;
                assemblies.push(ass)
            }
        }
        Ok(Self {
            assemblies,
            fontforge,
        })
    }
}

impl PrebuildManifest {
    pub fn process(&self, info: &PrebuildArgs) -> Result<()> {
        let mut watched = HashSet::new();
        watched.insert("Cargo.toml".to_string());
        watched.insert("src".to_string());

        let mut assembly_names = HashSet::new();

        if let Some(ff) = self.fontforge.as_ref() {
            ff.process(info)?;
            watched.insert(ff.file.to_string());
        }
        let mut generator = Generator::default();

        // go through all named assemblies
        for assembly in &self.assemblies {
            let Some(name) = assembly.name.as_ref() else {
                continue;
            };
            assembly_names.insert(name.to_string());
            if info.profile == assembly.profile {
                let change = assembly.process(info, &mut generator, name, true)?;
                watched.extend(change.into_iter());
            }
        }

        // go through wildcard assemblies
        for assembly in self.assemblies.iter().filter(|a| a.name.is_none()) {
            for name in &assembly_names {
                if info.profile == assembly.profile {
                    let change = assembly.process(info, &mut generator, name, false)?;
                    watched.extend(change.into_iter());
                }
            }
        }
        generator.write(info)?;
        for change in watched {
            println!("cargo::rerun-if-changed={}", change);
        }
        Ok(())
    }
}
