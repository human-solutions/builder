mod out;
mod sass;

use crate::{config::Assembly, Manifest, RuntimeInfo};
use anyhow::Result;
use fs_err as fs;

impl Manifest {
    pub fn process(&self, info: &RuntimeInfo) -> Result<()> {
        for assembly in &self.assemblies {
            if info.profile == assembly.profile {
                assembly.process(info)?;
            }
        }
        Ok(())
    }
}

impl Assembly {
    pub fn process(&self, info: &RuntimeInfo) -> Result<()> {
        let site_dir = info.site_dir(&self.name);
        if site_dir.exists() {
            fs::remove_dir_all(&site_dir)?;
        }
        fs::create_dir_all(&site_dir)?;
        println!("site: {site_dir}");

        for sass in &self.sass {
            let css = sass.process(info)?;
            let filename = sass.file.file_name().unwrap();
            sass.out.write_file(css.into_bytes(), &site_dir, filename)?;
        }
        Ok(())
    }
}
