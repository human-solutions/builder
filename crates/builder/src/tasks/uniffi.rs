use std::fmt;

use anyhow::Result;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use uniffi_bindgen::{
    bindings::{KotlinBindingGenerator, SwiftBindingGenerator},
    generate_external_bindings,
};

use crate::{anyhow::Context, generate::Output};

use super::Config;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) enum UniffiLanguage {
    #[default]
    #[serde(rename = "kotlin")]
    Kotlin,
    #[serde(rename = "swift")]
    Swift,
}

impl fmt::Display for UniffiLanguage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniffiLanguage::Kotlin => write!(f, "Kotlin"),
            UniffiLanguage::Swift => write!(f, "Swift"),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct UniffiParams {
    #[serde(rename = "udl-path")]
    pub udl_path: Utf8PathBuf,
    pub language: UniffiLanguage,
    pub out: Output,
}

impl UniffiParams {
    pub fn process(&self, config: &Config) -> Result<()> {
        let out_folder = self.out.folder.as_deref().unwrap_or("".into());
        let out_dir = config.site_dir("uniffi").join(out_folder);
        let udl_file = config.args.dir.join(&self.udl_path);

        match self.language {
            UniffiLanguage::Kotlin => generate_external_bindings(
                &KotlinBindingGenerator,
                &udl_file,
                None::<&Utf8PathBuf>,
                Some(out_dir),
                None::<&Utf8PathBuf>,
                Some(&config.package_name),
                true,
            ),
            UniffiLanguage::Swift => generate_external_bindings(
                &SwiftBindingGenerator,
                &self.udl_path,
                None::<&Utf8PathBuf>,
                Some(out_dir),
                None::<&Utf8PathBuf>,
                Some(&config.package_name),
                true,
            ),
        }
        .context(format!(
            "Failed to generate {} bindings for {}",
            self.language, config.package_name
        ))
    }
}
