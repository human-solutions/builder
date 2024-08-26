use crate::anyhow::Result;
use crate::ext::RustNaming;
use crate::Config;
use fs_err as fs;

use super::Asset;

const MODULE_FILE: &str = "generated_assets.rs";
const MODULE: &str = "GeneratedAssets";

#[derive(Default)]
pub struct Generator {
    assets: Vec<Asset>,
}

impl Generator {
    pub fn watched(&self) -> String {
        "gen".to_string()
    }

    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }
    pub fn write(&self, info: &Config) -> Result<()> {
        self.write_assembly(info, &self.assets)
    }

    pub fn write_assembly(&self, info: &Config, assets: &[Asset]) -> Result<()> {
        let text = self.text(assets);
        let dir = info.args.dir.join("gen");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        let path = dir.join(MODULE_FILE).with_extension("rs");
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

    pub fn text(&self, assets: &[Asset]) -> String {
        let constants = self.constants(assets);
        let matching = self.match_list(assets);
        format!(
            r#"
/// This is a generated file. Do not edit. It is updated depending on the build profile used (i.e. dev, release).
/// Instead it should be included with an include! macro: `include!("../gen/{MODULE_FILE}.rs");`
pub mod {MODULE} {{
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
