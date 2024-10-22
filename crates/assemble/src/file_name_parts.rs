use std::fmt::Display;

use crate::mime::mime_from_ext;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Compression {
    Brotli,
    Gzip,
    Uncompressed,
}

impl Compression {
    fn from(s: &str) -> Self {
        match s {
            "br" => Self::Brotli,
            "gzip" => Self::Gzip,
            _ => panic!("Invalid compression: {}", s),
        }
    }
}

impl Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Brotli => f.write_str("br"),
            Self::Gzip => f.write_str("gzip"),
            Self::Uncompressed => f.write_str(""),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileNameParts<'s> {
    pub name: &'s str,
    pub ext: &'s str,
    pub checksum: Option<&'s str>,
    pub compression: Compression,
    pub mime: &'static str,
}

impl Display for FileNameParts<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(checksum) = self.checksum {
            write!(f, "{}=", checksum)?;
        }
        write!(f, "{}.{}", self.name, self.ext)?;
        if self.compression != Compression::Uncompressed {
            write!(f, ".{}", self.compression)?;
        }
        Ok(())
    }
}

impl<'s> FileNameParts<'s> {
    pub fn from(filename: &'s str) -> Self {
        let parts = filename.split('.').collect::<Vec<_>>();
        let (name, ext, compression) = if parts.len() == 2 {
            (parts[0], parts[1], Compression::Uncompressed)
        } else if parts.len() == 3 {
            (parts[0], parts[1], Compression::from(parts[2]))
        } else {
            panic!("Invalid file name: {}", filename);
        };
        let name_chars = name.chars().collect::<Vec<_>>();
        let mime = mime_from_ext(ext);
        if name_chars.len() >= 12 && name_chars[11] == '=' {
            Self {
                name: &name[12..],
                ext,
                mime,
                checksum: Some(&name[..12]),
                compression,
            }
        } else {
            Self {
                name,
                ext,
                mime,
                checksum: None,
                compression,
            }
        }
    }
}

#[test]
fn filenamecomponents() {
    let components = FileNameParts::from("test.css");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: None,
            compression: Compression::Uncompressed
        }
    );

    let components = FileNameParts::from("test.css.gzip");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: None,
            compression: Compression::Gzip
        }
    );

    let components = FileNameParts::from("test.css.br");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: None,
            compression: Compression::Brotli
        }
    );

    let components = FileNameParts::from("1234567890a=test.css");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: Some("1234567890a="),
            compression: Compression::Uncompressed
        }
    );

    let components = FileNameParts::from("1234567890a=test.css.gzip");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: Some("1234567890a="),
            compression: Compression::Gzip
        }
    );

    let components = FileNameParts::from("1234567890a=test.css.br");
    assert_eq!(
        components,
        FileNameParts {
            name: "test",
            ext: "css",
            mime: "text/css",
            checksum: Some("1234567890a="),
            compression: Compression::Brotli
        }
    );
}
