use std::collections::HashMap;

use crate::anyhow::Result;
use crate::{ext::RustNaming, prebuild::PrebuildArgs};
use fs_err as fs;

use super::Asset;

#[derive(Default)]
pub struct Generator {
    assets: HashMap<String, Vec<Asset>>,
}

impl Generator {
    pub fn watched(&self) -> String {
        "gen".to_string()
    }

    pub fn add_asset(&mut self, assembly: &str, asset: Asset) {
        self.assets
            .entry(assembly.to_string())
            .or_default()
            .push(asset);
    }
    pub fn write(&self, info: &PrebuildArgs) -> Result<()> {
        for (module, assets) in &self.assets {
            self.write_assembly(module, info, assets)?;
        }
        Ok(())
    }

    pub fn write_assembly(
        &self,
        module: &str,
        info: &PrebuildArgs,
        assets: &[Asset],
    ) -> Result<()> {
        let module = module.to_rust_module();
        let text = self.text(&module, assets);
        let dir = info.manifest_dir.join("gen");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        let path = dir.join(module).with_extension("rs");
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

    pub fn text(&self, module: &str, assets: &[Asset]) -> String {
        let constants = self.constants(assets);
        let matching = self.match_list(assets);
        format!(
            r#"
/// This is a generated file. Do not edit. It is updated depending on the build profile used (i.e. dev, release).
/// Instead it should be included with an include! macro: `include!("../gen/{module}.rs");`
pub mod {module} {{
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
