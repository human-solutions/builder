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
            fs::remove_file(&path).unwrap();
        }
    }
}
