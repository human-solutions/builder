use cargo_metadata::{Metadata, Package};

pub trait MetadataExt {
    fn local_dependency_names(&self) -> impl Iterator<Item = &str>;
    fn local_dependency_packages(&self) -> impl Iterator<Item = &Package>;
    fn library_name(&self) -> Option<String>;
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

    fn library_name(&self) -> Option<String> {
        self.root_package().and_then(|root| {
            for target in &root.targets {
                if target.is_lib() {
                    return Some(target.name.clone());
                }
            }
            None
        })
    }
}
