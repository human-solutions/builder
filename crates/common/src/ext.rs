use camino::{Utf8DirEntry, Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::{collections::VecDeque, time::SystemTime};

pub trait RustNaming {
    fn to_rust_module(&self) -> String;
    fn to_rust_const(&self) -> String;
    fn to_camel_case(&self) -> String;
}

impl RustNaming for str {
    fn to_rust_module(&self) -> String {
        self.replace('-', "_")
    }

    fn to_rust_const(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for (i, char) in self.chars().enumerate() {
            if char == '.' {
                s.push('_');
                continue;
            } else if char == '_' {
                // allowed
            } else if !char.is_ascii_alphanumeric() {
                panic!("Only ascii chars and '.' allowed in rust constant names, not {char}")
            }
            if char.is_ascii_uppercase() && i != 0 {
                s.push('_');
                s.push(char);
            } else {
                s.push(char.to_ascii_uppercase());
            }
        }
        s
    }

    fn to_camel_case(&self) -> String {
        let mut s = String::with_capacity(self.len());
        let mut uppercase = true;
        for char in self.chars() {
            if s.is_empty() && (char.is_ascii_digit() || char == '-' || char == '.' || char == '_')
            {
                continue;
            } else if char == '.' || char == '_' || char == '-' {
                uppercase = true;
                continue;
            } else if char.is_ascii_alphanumeric() {
                if uppercase {
                    s.push(char.to_ascii_uppercase());
                    uppercase = false;
                } else {
                    s.push(char);
                }
            }
        }
        s
    }
}

pub trait Utf8PathExt {
    fn push_ext(&self, ext: &str) -> Utf8PathBuf;
    fn ls_files(&self) -> Vec<Utf8PathBuf>;
    fn ls_files_matching<P: Fn(&Self) -> bool>(&self, predicate: P) -> Vec<Utf8PathBuf>;
    fn ls_dirs_matching<P: Fn(&Utf8Path) -> bool>(&self, predicate: P) -> Vec<Utf8PathBuf>;
    fn relative_to(&self, base: &Utf8Path) -> Result<Utf8PathBuf, String>;
    fn create_dir_if_missing(&self) -> std::io::Result<()>;
    /// Returns the modification time of the file or
    /// None if the file does not exist.
    fn mtime(&self) -> Option<SystemTime>;
}

impl Utf8PathExt for Utf8Path {
    fn mtime(&self) -> Option<SystemTime> {
        self.metadata().ok().map(|md| md.modified().unwrap())
    }

    fn create_dir_if_missing(&self) -> std::io::Result<()> {
        if !self.exists() {
            fs::create_dir_all(self)
        } else {
            Ok(())
        }
    }

    fn push_ext(&self, ext: &str) -> Utf8PathBuf {
        let mut s = self.to_string();
        s.push('.');
        s.push_str(ext);
        Utf8PathBuf::from(s)
    }
    fn ls_files(&self) -> Vec<Utf8PathBuf> {
        files_matching(self, &|_| true)
    }

    fn ls_files_matching<P: Fn(&Self) -> bool>(&self, predicate: P) -> Vec<Utf8PathBuf> {
        files_matching(self, &predicate)
    }

    fn ls_dirs_matching<P: Fn(&Utf8Path) -> bool>(&self, predicate: P) -> Vec<Utf8PathBuf> {
        dirs_matching(self, &predicate)
    }

    fn relative_to(&self, base: &Utf8Path) -> Result<Utf8PathBuf, String> {
        let mut base = base.components();
        let mut self_components = self.components();
        while let Some(base) = base.next() {
            if self_components.next() != Some(base) {
                return Err(format!("{self} is not a subpath of {base}"));
            }
        }
        Ok(Utf8PathBuf::from(
            self_components.collect::<Utf8PathBuf>().to_string(),
        ))
    }
}

fn dirs_matching<P: Fn(&Utf8Path) -> bool>(path: &Utf8Path, predicate: &P) -> Vec<Utf8PathBuf> {
    let mut entries: VecDeque<Utf8DirEntry> =
        path.read_dir_utf8().unwrap().map(|e| e.unwrap()).collect();
    let mut dirs = Vec::new();
    while let Some(entry) = entries.pop_front() {
        let filetype = entry.file_type().unwrap();
        if filetype.is_dir() {
            if predicate(&entry.path()) {
                dirs.push(entry.path().to_path_buf());
            }
            entries.extend(entry.path().read_dir_utf8().unwrap().map(|e| e.unwrap()));
        }
    }
    dirs
}

fn files_matching<P: Fn(&Utf8Path) -> bool>(path: &Utf8Path, predicate: &P) -> Vec<Utf8PathBuf> {
    let mut entries: VecDeque<Utf8DirEntry> =
        path.read_dir_utf8().unwrap().map(|e| e.unwrap()).collect();
    let mut files = Vec::new();
    while let Some(entry) = entries.pop_front() {
        let filetype = entry.file_type().unwrap();
        if filetype.is_file() && predicate(&entry.path()) {
            files.push(entry.path().to_path_buf());
        }
        if filetype.is_dir() {
            entries.extend(entry.path().read_dir_utf8().unwrap().map(|e| e.unwrap()));
        }
    }
    files
}
