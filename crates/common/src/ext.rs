use crate::{debug, warn};
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
            if char == '.' || char == '-' {
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

pub trait StringExt {
    fn prefixed(&self, ch: char) -> String;
    fn postfixed(&self, ch: char) -> String;
}

impl StringExt for str {
    fn prefixed(&self, ch: char) -> String {
        format!("{ch}{self}")
    }
    fn postfixed(&self, ch: char) -> String {
        format!("{self}{ch}")
    }
}

pub trait OptStringExt {
    fn prefixed_or_default(&self, ch: char) -> String;
    fn postfixed_or_default(&self, ch: char) -> String;
}

impl OptStringExt for Option<String> {
    fn prefixed_or_default(&self, ch: char) -> String {
        self.as_ref().map(|s| s.prefixed(ch)).unwrap_or_default()
    }
    fn postfixed_or_default(&self, ch: char) -> String {
        self.as_ref().map(|s| s.postfixed(ch)).unwrap_or_default()
    }
}

impl OptStringExt for Option<&str> {
    fn prefixed_or_default(&self, ch: char) -> String {
        self.map(|s| s.prefixed(ch)).unwrap_or_default()
    }
    fn postfixed_or_default(&self, ch: char) -> String {
        self.map(|s| s.postfixed(ch)).unwrap_or_default()
    }
}

pub trait Utf8PathExt {
    fn push_ext(&self, ext: &str) -> Utf8PathBuf;
    fn ls(&self) -> impl Iterator<Item = Utf8PathBuf>;
    fn ls_recursive(&self) -> impl Iterator<Item = Utf8PathBuf>;
    fn remove_dir_content_matching<P: Fn(&Utf8Path) -> bool>(
        &self,
        predicate: P,
    ) -> std::io::Result<()>;

    fn relative_to(&self, base: &Utf8Path) -> Result<Utf8PathBuf, String>;
    fn create_dir_if_missing(&self) -> std::io::Result<()>;
    /// Returns the modification time of the file or
    /// None if the file does not exist.
    fn mtime(&self) -> Option<SystemTime>;
}

impl Utf8PathExt for Utf8Path {
    fn remove_dir_content_matching<P: Fn(&Utf8Path) -> bool>(
        &self,
        predicate: P,
    ) -> std::io::Result<()> {
        if !self.exists() {
            warn!("remove_dir_content_matching: path does not exist: {self}");
            return Ok(());
        }
        for entry in self.read_dir_utf8()? {
            let entry = entry?;
            let path = entry.path();
            if predicate(&path) {
                if path.is_dir() {
                    debug!("removing dir: {path}");
                    fs::remove_dir_all(path)?;
                } else {
                    debug!("removing file: {path}");
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    fn ls_recursive(&self) -> impl Iterator<Item = Utf8PathBuf> {
        let mut entries: VecDeque<Utf8DirEntry> =
            self.read_dir_utf8().unwrap().map(|e| e.unwrap()).collect();
        std::iter::from_fn(move || {
            while let Some(entry) = entries.pop_front() {
                let filetype = entry.file_type().unwrap();
                if filetype.is_dir() {
                    entries.extend(entry.path().read_dir_utf8().unwrap().map(|e| e.unwrap()));
                }
                return Some(entry.path().to_path_buf());
            }
            None
        })
    }

    fn ls(&self) -> impl Iterator<Item = Utf8PathBuf> {
        self.read_dir_utf8()
            .unwrap()
            .map(|e| e.unwrap().path().to_path_buf())
    }

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
