use crate::config::Sass;

#[derive(Debug)]
pub struct Asset {
    /// the url used to access the asset
    pub url: String,
    /// the name of the asset
    pub name: String,
    pub encodings: Vec<String>,
}

impl Asset {
    pub fn from_sass(sass: &Sass, checksum: Option<String>) -> Self {
        let folder = if let Some(folder) = sass.out.folder.as_ref() {
            format!("/{folder}")
        } else {
            "".to_string()
        };
        let filename = sass.file.file_name().unwrap();
        let url = format!("{folder}/{}{filename}", checksum.unwrap_or_default());
        Self {
            url,
            name: sass.file.file_name().unwrap().to_string(),
            encodings: sass.out.encodings(),
        }
    }

    pub fn quoted_encoding_list(&self) -> (usize, String) {
        let count = self.encodings.len();
        let encodings = self
            .encodings
            .iter()
            .map(|e| format!(r#""{}""#, e))
            .collect::<Vec<_>>()
            .join(", ");
        (count, encodings)
    }
}
