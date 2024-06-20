use fs_err as fs;
use toml_edit::{DocumentMut, Item, TableLike};

use anyhow::{bail, Context, Result};

use crate::RuntimeInfo;

use super::Sass;

#[derive(Debug)]
pub struct Manifest {
    pub code_gen_dir: Option<String>,
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

        let mut code_gen_dir = None;
        let mut assemblies = Vec::new();
        for (name, value) in names {
            if name == "code-gen-dir" {
                code_gen_dir = Some(value.to_string());
            } else {
                for (profile, toml) in value.as_table().unwrap() {
                    let ass = Assembly::try_parse(name, profile, toml)?;
                    assemblies.push(ass)
                }
            }
        }
        Ok(Self {
            code_gen_dir,
            assemblies,
        })
    }
}

#[derive(Debug)]
pub struct Assembly {
    pub name: String,
    pub profile: String,
    pub sass: Vec<Sass>,
}

impl Assembly {
    fn try_parse(name: &str, profile: &str, toml: &Item) -> Result<Self> {
        let name = name.to_string();
        let profile = profile.to_string();
        let table = toml.as_table().context("no content")?;

        let mut sass = Vec::new();
        for (process, toml) in table {
            match process {
                "sass" => {
                    sass =
                        parse_vec(toml, Sass::try_parse).context("Could not parse sass values")?;
                }
                _ => bail!("Invalid processing type: {name}"),
            }
        }
        Ok(Self {
            name,
            profile,
            sass,
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
