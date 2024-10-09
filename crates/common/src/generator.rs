use std::collections::HashMap;

use crate::{asset::Asset, Cli};
use camino::Utf8PathBuf;
use common::RustNaming;
use fs_err as fs;

impl Cli {
    pub fn input_dir_name(&self) -> String {
        self.input_dir.iter().last().unwrap().to_string()
    }

    pub fn url(&self, checksum: Option<String>) -> String {
        let ext = &self.file_extension;
        let name = self.input_dir_name();
        let sum = checksum.as_deref().unwrap_or_default();
        format!("{sum}{name}.{ext}")
    }
}

#[derive(Default)]
pub struct Generator {
    assets: HashMap<Utf8PathBuf, Vec<Asset>>,
}

impl Generator {
    pub fn add_asset(&mut self, asset: Asset, module_path: Utf8PathBuf) {
        self.assets.entry(module_path).or_default().push(asset);
    }

    pub fn write(&self, cli: &Cli) {
        for (module_path, assets) in &self.assets {
            self.write_assembly(cli, assets, module_path)
        }
    }

    fn write_assembly(&self, cli: &Cli, assets: &[Asset], module_path: &Utf8PathBuf) {
        log::info!("Writing assembly file for {:?}", module_path);

        let module_name = {
            let m = module_path.file_stem().unwrap().to_camel_case();
            format!("{m}Assets")
        };

        let file_name = module_path.file_stem().unwrap().to_rust_module();

        let text = self.text(assets, &module_name, &file_name);

        if cli.generate_code.exists() {
            let content = fs::read_to_string(&cli.generate_code).unwrap();
            if content == text {
                return;
            }
        } else if let Some(parent_dir) = &cli.generate_code.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).unwrap();
            }
        }
        fs::write(&cli.generate_code, text).unwrap();
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
