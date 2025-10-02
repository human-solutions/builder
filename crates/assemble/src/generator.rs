use crate::asset_ext::AssetExt;
use common::mime::mime_from_ext;
use common::{RustNaming, site_fs::Asset};

pub fn generate_code(assets: &[Asset]) -> String {
    let statics = static_vars(assets);
    let constants = url_constants(assets);

    let matching = match_list(assets);
    format!(
        r#"
// This is a generated file. Do not edit.
use icu_locid::{{langid, LanguageIdentifier}};

{constants}

{statics}

{asset_rs}

impl Asset {{

    pub fn from_url(url: &str) -> Option<Asset> {{
        match url {{
    {matching}
            _ => None,
        }}
    }}
}}
"#,
        asset_rs = include_str!("asset_incl.rs")
    )
}

fn match_list(assets: &[Asset]) -> String {
    let mut matches = vec![];
    for asset in assets {
        let const_name = asset.name.to_rust_const();
        let encodings = if asset.encodings.is_empty() {
            panic!("Asset {} has no encodings", asset.name);
        } else {
            format!("&{const_name}_ENC")
        };
        let langs = if asset.translations.is_empty() {
            "None".to_string()
        } else {
            format!("Some(&{const_name}_LANGS)")
        };
        let mime = mime_from_ext(&asset.ext);
        matches.push(format!(
            r#"        {const_name}_URL => Some(Asset {{
                mime: "{mime}",
                langs: {langs},
                encodings: {encodings},
            }}),"#,
        ));
    }
    matches.sort();
    matches.join("\n")
}

fn url_constants(assets: &[Asset]) -> String {
    let mut constants = assets
        .iter()
        .map(|asset| asset.url_const())
        .collect::<Vec<_>>();
    constants.sort();
    constants.join("\n")
}

fn static_vars(assets: &[Asset]) -> String {
    let mut constants = vec![];
    for asset in assets {
        let name = asset.name.to_rust_const();

        let (count, encodings) = asset.quoted_encoding_list();
        if count > 0 {
            constants.push(format!(
                r#"pub static {name}_ENC: [&str; {count}] = [{encodings}];"#,
                count = asset.encodings.len()
            ));
        }

        let (count, langs) = asset.langid_list();
        if count > 0 {
            constants.push(format!(
                r#"pub static {name}_LANGS: [LanguageIdentifier; {count}] = [{langs}];"#,
                count = asset.translations.len()
            ));
        }
    }
    constants.sort();
    constants.join("\n")
}
