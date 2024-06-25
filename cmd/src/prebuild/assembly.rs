use super::{File, Localized, PrebuildArgs, Sass};
use crate::{
    generate::{Asset, Generator},
    util::parse_vec,
};
use anyhow::{bail, Context, Result};
use fs_err as fs;
use toml_edit::Item;

#[derive(Debug)]
pub struct Assembly {
    pub name: Option<String>,
    pub profile: String,
    pub sass: Vec<Sass>,
    pub localized: Vec<Localized>,
    pub files: Vec<File>,
}

impl Assembly {
    pub fn try_parse(name: &str, profile: &str, toml: &Item) -> Result<Self> {
        let name = name.to_string();
        let name = (name != "*").then_some(name);

        let profile = profile.to_string();
        let table = toml.as_table().context("no content")?;

        let mut sass = Vec::new();
        let mut localized = Vec::new();
        let mut files = Vec::new();

        for (process, toml) in table {
            match process {
                "sass" => {
                    sass =
                        parse_vec(toml, Sass::try_parse).context("Could not parse sass values")?;
                }
                "localized" => {
                    localized = parse_vec(toml, Localized::try_parse)
                        .context("Could not parse localized values")?
                }
                "files" => {
                    files =
                        parse_vec(toml, File::try_parse).context("Could not parse file value")?;
                }
                _ => bail!("Invalid processing type: {process}"),
            }
        }
        Ok(Self {
            name,
            profile,
            sass,
            localized,
            files,
        })
    }

    pub fn process(
        &self,
        info: &PrebuildArgs,
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
        }
        let mut watched = vec![generator.watched()];

        for sass in &self.sass {
            let css = sass.process(info)?;
            let filename = sass.file.file_name().unwrap();
            let hash = sass.out.write_file(css.into_bytes(), &site_dir, filename)?;

            generator.add_asset(name, Asset::from_sass(sass, hash));
            watched.push(sass.watched());
        }
        for localized in &self.localized {
            let variants = localized.process(info)?;
            let localizations = variants.iter().map(|(lang, _)| lang.clone()).collect();

            let filename = localized.path.iter().last().unwrap();
            let ext = &localized.file_ext;
            let hash = localized
                .out
                .write_localized(&site_dir, filename, ext, variants)?;

            generator.add_asset(name, Asset::from_localized(localized, hash, localizations));
            watched.push(localized.path.to_string());
        }
        for file in &self.files {
            let path = info.manifest_dir.join(&file.path);
            let contents = fs::read(&path)?;
            let filename = file.path.file_name().unwrap();
            let hash = file.out.write_file(contents, &site_dir, filename)?;

            generator.add_asset(name, Asset::from_file(file, hash));
            watched.push(file.path.to_string());
        }

        Ok(watched)
    }
}
