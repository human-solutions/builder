use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{Context, Result};
use cargo_metadata::{camino::Utf8PathBuf, Metadata, MetadataCommand, Package};
use serde::Serialize;
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Serialize)]
struct Output {
    pub content: Value,
}

impl Output {
    fn new(content: Value) -> Self {
        Self { content }
    }
}

#[derive(Serialize)]
pub struct Outputs(Vec<Output>);

impl Outputs {
    pub fn gather(dir: &Utf8PathBuf) -> Result<Self> {
        let mut added = Vec::new();
        let mut outputs = Vec::new();

        gather_deps(&mut outputs, &dir.join("Cargo.toml"), &mut added)?;

        Ok(Self(outputs))
    }
}

fn gather_deps(
    outputs: &mut Vec<Output>,
    manifest_path: &Utf8PathBuf,
    added: &mut Vec<PathBuf>,
) -> Result<()> {
    let metadata = MetadataCommand::new().manifest_path(manifest_path).exec()?;

    let target_dir = &metadata.target_directory;

    for package in metadata.local_dependency_packages() {
        gather_files(outputs, target_dir, &package.name, added)?;
        gather_deps(outputs, &package.manifest_path, added)?;
    }

    Ok(())
}

fn gather_files(
    outputs: &mut Vec<Output>,
    target_dir: &Utf8PathBuf,
    package_name: &str,
    added: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in WalkDir::new(target_dir.join(format!("builder/{}", package_name))) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file()
                && entry.file_name() == "Output.yaml"
                && !added.contains(&entry.path().to_owned())
            {
                let file = File::open(entry.path()).context(format!(
                    "Failed to open output file '{}'",
                    entry.path().to_str().unwrap_or("")
                ))?;
                let reader = BufReader::new(file);
                let contents: Value = serde_yaml::from_reader(reader).context(format!(
                    "Failed to read output file '{}'",
                    entry.path().to_str().unwrap_or("")
                ))?;

                outputs.push(Output::new(contents));
                added.push(entry.path().to_owned());
            }
        } else {
            // needed to avoid clippy warning
            continue;
        }
    }

    Ok(())
}

pub trait MetadataExt {
    fn local_dependency_names(&self) -> impl Iterator<Item = &str>;
    fn local_dependency_packages(&self) -> impl Iterator<Item = &Package>;
}

impl MetadataExt for Metadata {
    fn local_dependency_names(&self) -> impl Iterator<Item = &str> {
        let root_pack = self.root_package().unwrap();
        root_pack
            .dependencies
            .iter()
            .filter(|dep| dep.path.is_some())
            .map(|dep| dep.name.as_str())
    }

    fn local_dependency_packages(&self) -> impl Iterator<Item = &Package> {
        let names = self.local_dependency_names().collect::<Vec<_>>();
        self.packages
            .iter()
            .filter(move |pkg| names.contains(&pkg.name.as_str()))
    }
}
