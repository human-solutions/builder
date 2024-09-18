use std::{env, fmt};

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
        const DEFAULT_PROFILE: &str = "debug";

        let out_folder = self.out.folder.as_deref().unwrap_or("".into());
        let out_dir = config.site_dir("uniffi").join(out_folder);

        let udl_file = config.args.dir.join(self.udl_path.as_str());

        let profile = if config.args.profile.is_empty() {
            DEFAULT_PROFILE.to_string()
        } else {
            env::var("OUT_DIR")
                .context("Failed to get OUT_DIR environment variable")?
                .split(std::path::MAIN_SEPARATOR)
                .nth_back(3)
                .unwrap_or(DEFAULT_PROFILE)
                .to_string()
        };

        let profile = if profile == "release" {
            "lib-release".to_owned()
        } else {
            profile
        };

        let is_mac = cfg!(target_os = "macos");
        let ext = if is_mac { "dylib" } else { "so" };

        let library_file = config
            .target_dir
            .join(format!("{}/lib{}.{}", profile, config.package_name, ext));

        match self.language {
            UniffiLanguage::Kotlin => generate_external_bindings(
                &KotlinBindingGenerator,
                &udl_file,
                None::<&Utf8PathBuf>,
                Some(out_dir),
                Some(library_file),
                Some(&config.package_name),
                true,
            ),
            UniffiLanguage::Swift => generate_external_bindings(
                &SwiftBindingGenerator,
                &udl_file,
                None::<&Utf8PathBuf>,
                Some(out_dir),
                Some(library_file),
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
