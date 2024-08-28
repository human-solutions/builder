use std::collections::HashMap;

use crate::anyhow::Result;
use crate::ext::RustNaming;
use crate::tasks::Config;
use anyhow::Context;
use camino::Utf8PathBuf;
use fs_err as fs;

use super::Asset;

const MODULE_DIR: &str = "gen";
const MODULE_FILE: &str = "generated_assets";
const MODULE: &str = "Generated";

#[derive(Default)]
pub struct Generator {
    assets: HashMap<Option<Utf8PathBuf>, Vec<Asset>>,
}

impl Generator {
    pub fn watched(&self) -> String {
        "gen".to_string()
    }

    pub fn add_asset(&mut self, asset: Asset, module_path: Option<Utf8PathBuf>) {
        self.assets.entry(module_path).or_default().push(asset);
    }
    pub fn write(&self, config: &Config) -> Result<()> {
        for (module_path, assets) in &self.assets {
            self.write_assembly(config, assets, module_path)?;
        }

        Ok(())
    }

    fn write_assembly(
        &self,
        config: &Config,
        assets: &[Asset],
        module_path: &Option<Utf8PathBuf>,
    ) -> Result<()> {
        let module_name = {
            let m = if let Some(p) = module_path {
                p.file_stem()
                    .context(format!("Invalid module path: {p}"))?
                    .to_camel_case()
            } else {
                MODULE.to_owned()
            };
            format!("{m}Assets")
        };

        let file_name = if let Some(p) = module_path {
            p.file_stem()
                .context(format!("Failed to get module file name: {p}"))?
                .to_rust_module()
        } else {
            MODULE_FILE.to_owned()
        };

        let dir = if let Some(p) = module_path {
            config.args.dir.join(
                p.parent()
                    .context(format!("Failed to get module parent path : {p}"))?,
            )
        } else {
            config.args.dir.join(MODULE_DIR)
        };

        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        let text = self.text(assets, &module_name, &file_name);

        let path = dir.join(file_name).with_extension("rs");
        let changed = if path.exists() {
            fs::read_to_string(&path)? != text
        } else {
            true
        };

        if changed {
            fs::write(&path, text)?;
        }
        Ok(())
    }

    fn text(&self, assets: &[Asset], module_name: &str, file_name: &str) -> String {
        let constants = self.constants(assets);
        let matching = self.match_list(assets);
        format!(
            r#"
/// This is a generated file. Do not edit. It is updated depending on the build profile used (i.e. dev, release).
/// Instead it should be included with an include! macro: `include!("../gen/{file_name}.rs");`
pub mod {module_name} {{
    #![allow(dead_code)]
{constants}


    pub struct AssetOptions {{
        pub langs: Option<&'static [&'static str]>,
        pub encodings: Option<&'static [&'static str]>,
    }}

    pub fn localisations_and_compressions_for_url(url: &str) -> Option<AssetOptions> {{
       match url {{
{matching}
            _ => None,
        }}
    }}
}}
"#,
        )
    }

    fn match_list(&self, assets: &[Asset]) -> String {
        let mut matches = vec![];
        for asset in assets {
            let url = &asset.url;
            let const_name = asset.name.to_rust_const();
            let encodings = if asset.encodings.is_empty() {
                "None".to_string()
            } else {
                format!("Some(&{const_name}_ENC)")
            };
            let langs = if asset.localizations.is_empty() {
                "None".to_string()
            } else {
                format!("Some(&{const_name}_LANGS)")
            };
            matches.push(format!(
                r#"            "{url}" => Some(AssetOptions {{
                langs: {langs},
                encodings: {encodings},
            }}),"#,
            ));
        }
        matches.join("\n")
    }

    fn constants(&self, assets: &[Asset]) -> String {
        let mut constants = vec![];
        for asset in assets {
            let name = asset.name.to_rust_const();
            let url = &asset.url;
            constants.push(format!(r#"    pub const {name}_URL: &str = "{url}";"#,));

            let (count, encodings) = asset.quoted_encoding_list();
            if count > 0 {
                let count = asset.encodings.len();
                constants.push(format!(
                    r#"    pub const {name}_ENC: [&str; {count}] = [{encodings}];"#,
                ));
            }

            let (count, langs) = asset.quoted_lang_list();
            if count > 0 {
                let count = asset.localizations.len();
                constants.push(format!(
                    r#"    pub const {name}_LANGS: [&str; {count}] = [{langs}];"#,
                ));
            }
        }
        constants.join("\n")
    }
}
