use crate::asset::Asset;
use common::RustNaming;

pub fn generate_code(assets: &[Asset], url_prefix: &str) -> String {
    let constants = constants(assets, url_prefix);
    let matching = match_list(assets, url_prefix);
    format!(
        r#"
#![allow(dead_code)]
// This is a generated file. Do not edit. It is updated depending on the build profile used (i.e. dev, release).
{constants}

#[derive(Debug)]
pub struct AssetOptions {{
    pub langs: Option<&'static [&'static str]>,
    pub encodings: &'static [&'static str],
}}

pub fn localisations_and_compressions_for_url(url: &str) -> Option<AssetOptions> {{
    match url {{
{matching}
        _ => None,
    }}
}}
"#,
    )
}

fn match_list(assets: &[Asset], url_prefix: &str) -> String {
    let mut matches = vec![];
    for asset in assets {
        let url = &asset.url;
        let const_name = asset.name.to_rust_const();
        let encodings = if asset.encodings.is_empty() {
            panic!("Asset {} has no encodings", asset.name);
        } else {
            format!("&{const_name}_ENC")
        };
        let langs = if asset.localizations.is_empty() {
            "None".to_string()
        } else {
            format!("Some(&{const_name}_LANGS)")
        };
        matches.push(format!(
            r#"        "{url_prefix}{url}" => Some(AssetOptions {{
                langs: {langs},
                encodings: {encodings},
            }}),"#,
        ));
    }
    matches.join("\n")
}

fn constants(assets: &[Asset], url_prefix: &str) -> String {
    let mut constants = vec![];
    for asset in assets {
        let name = asset.name.to_rust_const();
        let url = &asset.url;
        constants.push(format!(
            r#"pub const {name}_URL: &str = "{url_prefix}{url}";"#,
        ));

        let (count, encodings) = asset.quoted_encoding_list();
        if count > 0 {
            let count = asset.encodings.len();
            constants.push(format!(
                r#"pub const {name}_ENC: [&str; {count}] = [{encodings}];"#,
            ));
        }

        let (count, langs) = asset.quoted_lang_list();
        if count > 0 {
            let count = asset.localizations.len();
            constants.push(format!(
                r#"pub const {name}_LANGS: [&str; {count}] = [{langs}];"#,
            ));
        }
    }
    constants.join("\n")
}
