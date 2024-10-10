use camino::{Utf8Path, Utf8PathBuf};

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
}

impl Utf8PathExt for Utf8Path {
    fn push_ext(&self, ext: &str) -> Utf8PathBuf {
        let mut s = self.to_string();
        s.push('.');
        s.push_str(ext);
        Utf8PathBuf::from(s)
    }
    fn ls_files(&self) -> Vec<Utf8PathBuf> {
        let mut entries = self.read_dir_utf8().unwrap();
        let mut files = Vec::new();
        while let Some(Ok(entry)) = entries.next() {
            if entry.file_type().unwrap().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        files
    }
}
