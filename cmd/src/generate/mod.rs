mod asset;

use crate::{ext::RustNaming, RuntimeInfo};
use anyhow::Result;
pub use asset::Asset;
use fs_err as fs;

#[derive(Default)]
pub struct Generator {
    assets: Vec<Asset>,
}

impl Generator {
    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }
    pub fn write(&self, module: &str, info: &RuntimeInfo) -> Result<()> {
        let module = module.to_rust_module();
        let text = self.text(&module);
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

    pub fn text(&self, module: &str) -> String {
        let constants = self.constants();
        let matching = self.match_list();
        format!(
            r#"
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

    fn match_list(&self) -> String {
        let mut matches = vec![];
        for asset in &self.assets {
            let url = &asset.url;
            let const_name = asset.name.to_rust_const();
            let encodings = if asset.encodings.is_empty() {
                "None".to_string()
            } else {
                format!("Some(&{const_name}_ENC)")
            };
            let langs = "None";
            matches.push(format!(
                r#"            "{url}" => Some(AssetOptions {{
                langs: {langs},
                encodings: {encodings},
            }}),"#,
            ));
        }
        matches.join("\n")
    }

    fn constants(&self) -> String {
        let mut constants = vec![];
        for asset in &self.assets {
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
        }
        constants.join("\n")
    }
}
