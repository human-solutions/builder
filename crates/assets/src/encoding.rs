use std::fmt::Display;

/// File encodings in order of preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Brotli,
    Gzip,
    /// uncompressed
    Identity,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Encoding {
    pub fn name(&self) -> &'static str {
        match self {
            Encoding::Brotli => "Brotli",
            Encoding::Gzip => "Gzip",
            Encoding::Identity => "Identity",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Brotli => "br",
            Encoding::Gzip => "gzip",
            Encoding::Identity => "",
        }
    }

    pub fn file_ending(&self) -> Option<&str> {
        match self {
            Encoding::Brotli => Some("br"),
            Encoding::Gzip => Some("gzip"),
            Encoding::Identity => None,
        }
    }

    /// Returns the preference order for this encoding.
    /// Lower numbers have higher preference.
    pub fn preference_order(&self) -> u8 {
        match self {
            Encoding::Brotli => 0,
            Encoding::Gzip => 1,
            Encoding::Identity => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_display() {
        assert_eq!(Encoding::Brotli.to_string(), "br");
        assert_eq!(Encoding::Gzip.to_string(), "gzip");
        assert_eq!(Encoding::Identity.to_string(), "");
    }

    #[test]
    fn test_encoding_name() {
        assert_eq!(Encoding::Brotli.name(), "Brotli");
        assert_eq!(Encoding::Gzip.name(), "Gzip");
        assert_eq!(Encoding::Identity.name(), "Identity");
    }

    #[test]
    fn test_file_ending() {
        assert_eq!(Encoding::Brotli.file_ending(), Some("br"));
        assert_eq!(Encoding::Gzip.file_ending(), Some("gzip"));
        assert_eq!(Encoding::Identity.file_ending(), None);
    }

    #[test]
    fn test_preference_order() {
        assert_eq!(Encoding::Brotli.preference_order(), 0);
        assert_eq!(Encoding::Gzip.preference_order(), 1);
        assert_eq!(Encoding::Identity.preference_order(), 2);
    }
}
