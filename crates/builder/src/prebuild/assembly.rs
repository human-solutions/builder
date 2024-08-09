use super::{File, Localized, Sass};
use crate::anyhow::Result;
use crate::generate::{Asset, Generator};
use crate::Config;
use fs_err as fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default, Serialize)]
#[serde(default)]
pub struct Assembly {
    #[serde(skip)]
    pub name: Option<String>,
    #[serde(skip)]
    pub target: String,
    #[serde(skip)]
    pub profile: String,

    pub sass: Vec<Sass>,
    pub localized: Vec<Localized>,
    pub files: Vec<File>,
}

impl Assembly {
    pub fn process(
        &self,
        info: &Config,
        generator: &mut Generator,
        name: &str,
        clean: bool,
    ) -> Result<Vec<String>> {
        let site_dir = info.site_dir(name);
        if clean {
            if site_dir.exists() {
                fs::remove_dir_all(&site_dir)?;
            }
            fs::create_dir_all(&site_dir)?;
            println!("Cleaned: {site_dir}");
        }
        let mut watched = vec![generator.watched()];

        for sass in &self.sass {
            log::info!("Processing sass assembly '{name}'");
            let css = sass.process(info)?;
            let filename = sass.file.file_name().unwrap().replace("scss", "css");
            let hash = sass.out.write_file(css.as_bytes(), &site_dir, &filename)?;

            generator.add_asset(name, Asset::from_sass(sass, hash));
            watched.push(sass.watched());
        }
        for localized in &self.localized {
            log::info!("Processing localized assembly '{name}'");
            let variants = localized.process(info)?;
            let localizations = variants.iter().map(|(lang, _)| lang.clone()).collect();

            let filename = localized.path.iter().last().unwrap();
            let ext = &localized.file_extension;
            let hash = localized
                .out
                .write_localized(&site_dir, filename, ext, variants)?;

            generator.add_asset(name, Asset::from_localized(localized, hash, localizations));
            watched.push(localized.path.to_string());
        }
        for file in &self.files {
            let path = info.args.dir.join(&file.path);
            let contents = fs::read(&path)?;
            let filename = file.path.file_name().unwrap();
            let hash = file.out.write_file(&contents, &site_dir, filename)?;

            generator.add_asset(name, Asset::from_file(file, hash));
            watched.push(file.path.to_string());
        }

        Ok(watched)
    }
}
