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
        watched.insert("Cargo.toml".to_string());
        watched.insert("src".to_string());

        let mut assembly_names = HashSet::new();

        if let Some(ff) = self.fontforge.as_ref() {
            ff.process(info)?;
            watched.insert(ff.file.to_string());
        }
        let mut generator = Generator::default();

        // go through all named assemblies
        for assembly in &self.assemblies {
            let Some(name) = assembly.name.as_ref() else {
                continue;
            };
            assembly_names.insert(name.to_string());
            if info.profile == assembly.profile {
                let change = assembly.process(info, &mut generator, name, true)?;
                watched.extend(change.into_iter());
            }
        }

        // go through wildcard assemblies
        for assembly in self.assemblies.iter().filter(|a| a.name.is_none()) {
            for name in &assembly_names {
                if info.profile == assembly.profile {
                    let change = assembly.process(info, &mut generator, name, false)?;
                    watched.extend(change.into_iter());
                }
            }
        }
        generator.write(info)?;
        for change in watched {
            println!("cargo::rerun-if-changed={}", change);
        }
        Ok(())
    }
}

impl Assembly {
    pub fn process(
        &self,
        info: &RuntimeInfo,
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
