use std::path::Path;

use fs_err as fs;

pub fn remove_content_of_dir<P: AsRef<Path>>(dir: P) {
    let dir = dir.as_ref();
    if !dir.exists() {
        return;
    }
    if !dir.is_dir() {
        panic!("Expected a directory, found a file: {:?}", dir);
    }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            log::info!("Removing {:?}", path);
            fs::remove_file(&path).unwrap();
        }
    }
}

pub fn remove_files_containing<P: AsRef<Path>>(dir: P, name: &str) {
    let dir = dir.as_ref();
    if !dir.exists() {
        return;
    }
    if !dir.is_dir() {
        panic!("Expected a directory, found a file: {:?}", dir);
    }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && filename(&path).contains(name) {
            log::info!("Removing {:?} because its name contains '{name}'", path);
            fs::remove_file(&path).unwrap();
        }
    }
}

fn filename(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}
