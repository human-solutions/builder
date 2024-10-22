use crate::asset::Asset;
use common::RustNaming;

pub fn generate_code(assets: &[Asset], url_prefix: &str) -> String {
    let constants = constants(assets, url_prefix);
    let matching = match_list(assets);
    format!(
        r#"
// This is a generated file. Do not edit.
{constants}

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
        let langs = if asset.localizations.is_empty() {
            "None".to_string()
        } else {
            format!("Some(&{const_name}_LANGS)")
        };
        let mime = &asset.mime;
        matches.push(format!(
            r#"        {const_name}_URL => Some(Asset {{
                mime: "{mime}",
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
        constants.push(asset.url_const(url_prefix));

        let (count, encodings) = asset.quoted_encoding_list();
        if count > 0 {
            constants.push(format!(
                r#"pub const {name}_ENC: [&str; {count}] = [{encodings}];"#,
                count = asset.encodings.len()
            ));
        }

        let (count, langs) = asset.quoted_lang_list();
        if count > 0 {
            constants.push(format!(
                r#"pub const {name}_LANGS: [&str; {count}] = [{langs}];"#,
                count = asset.localizations.len()
            ));
        }
    }
    constants.join("\n")
}
