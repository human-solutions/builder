mod fontforge;
mod localized;
mod out;
mod sass;

use std::collections::HashSet;

use crate::{
    config::Assembly,
    generate::{Asset, Generator},
    Manifest, RuntimeInfo,
};
use anyhow::Result;
use fs_err as fs;

impl Manifest {
    pub fn process(&self, info: &RuntimeInfo) -> Result<()> {
        let mut watched = HashSet::new();
        for assembly in &self.assemblies {
            if info.profile == assembly.profile {
                let change = assembly.process(info)?;
                watched.extend(change.into_iter());
            }
        }
        if let Some(ff) = self.fontforge.as_ref() {
            ff.process(info)?;
            watched.insert(ff.file.to_string());
        }
        for change in watched {
            println!("cargo::rerun-if-changed={}", change);
        }
        Ok(())
    }
}

impl Assembly {
    pub fn process(&self, info: &RuntimeInfo) -> Result<Vec<String>> {
        let site_dir = info.site_dir(&self.name);
        if site_dir.exists() {
            fs::remove_dir_all(&site_dir)?;
        }
        fs::create_dir_all(&site_dir)?;

        let mut generator = Generator::default();
        let mut watched = vec![generator.watched()];

        for sass in &self.sass {
            let css = sass.process(info)?;
            let filename = sass.file.file_name().unwrap();
            let hash = sass.out.write_file(css.into_bytes(), &site_dir, filename)?;

            generator.add_asset(Asset::from_sass(sass, hash));
            watched.push(sass.watched());
        }
        for localized in &self.localized {
            let variants = localized.process(info)?;
            let localizations = variants.iter().map(|(lang, _)| lang.clone()).collect();

            let filename = localized.path.iter().last().unwrap();
            let hash = localized
                .out
                .write_localized(&site_dir, filename, variants)?;

            generator.add_asset(Asset::from_localized(localized, hash, localizations));
            watched.push(localized.path.to_string());
        }
        generator.write(&self.name, info)?;

        Ok(watched)
    }
}
