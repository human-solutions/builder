use fs_err as fs;
use toml_edit::{DocumentMut, Item, TableLike};

use anyhow::{bail, Context, Result};

use crate::RuntimeInfo;

use super::{localized::Localized, Sass};

#[derive(Debug)]
pub struct Manifest {
    pub assemblies: Vec<Assembly>,
}

impl Manifest {
    pub fn try_parse(info: &RuntimeInfo) -> Result<Self> {
        let manifest_str = fs::read_to_string(info.manifest_dir.join("Cargo.toml"))?;
        let manifest = manifest_str.parse::<DocumentMut>()?;
        let val = &manifest
            .get("package")
            .context("Could not find package section in manifest")?
            .get("metadata")
            .context("Could not find package.metadata section in manifest")?
            .get("builder")
            .context("Could not find package.metadata.builder section in manifest")?;

        let names = val.as_table().context(
            "Could not find assembly name. Expected package.metadata.builder.<assembly>",
        )?;

        let mut assemblies = Vec::new();
        for (name, value) in names {
            for (profile, toml) in value.as_table().unwrap() {
                let ass = Assembly::try_parse(name, profile, toml)?;
                assemblies.push(ass)
            }
        }
        Ok(Self { assemblies })
    }
}

#[derive(Debug)]
pub struct Assembly {
    pub name: String,
    pub profile: String,
    pub sass: Vec<Sass>,
    pub localized: Vec<Localized>,
}

impl Assembly {
    fn try_parse(name: &str, profile: &str, toml: &Item) -> Result<Self> {
        let name = name.to_string();
        let profile = profile.to_string();
        let table = toml.as_table().context("no content")?;

        let mut sass = Vec::new();
        let mut localized = Vec::new();
        for (process, toml) in table {
            match process {
                "sass" => {
                    sass =
                        parse_vec(toml, Sass::try_parse).context("Could not parse sass values")?;
                }
                "localized" => {
                    localized = parse_vec(toml, Localized::try_parse)
                        .context("Could not parse localized values")?
                }
                _ => bail!("Invalid processing type: {process}"),
            }
        }
        Ok(Self {
            name,
            profile,
            sass,
            localized,
        })
    }
}

fn parse_vec<T, F: Fn(&dyn TableLike) -> Result<T>>(item: &Item, f: F) -> Result<Vec<T>> {
    let mut vals = Vec::new();
    if let Some(arr) = item.as_array() {
        for entry in arr {
            let table = entry
                .as_inline_table()
                .context("Expected an inline table")?;

            vals.push(f(table)?)
        }
    } else if let Some(arr_tbl) = item.as_array_of_tables() {
        for table in arr_tbl {
            vals.push(f(table)?)
        }
    } else {
        bail!("Expected an array of tables or an array")
    }
    Ok(vals)
}
