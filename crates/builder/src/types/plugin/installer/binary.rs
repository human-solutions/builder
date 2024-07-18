use std::fmt;

#[derive(Debug)]
pub enum BinStatus {
    Installed { version: String },
    NotInstalled,
}

impl fmt::Display for BinStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinStatus::Installed { version } => write!(f, "{version}"),
            BinStatus::NotInstalled => write!(f, "Not installed"),
        }
    }
}

#[derive(Debug)]
pub struct Binary<'a> {
    pub name: String,
    pub status: BinStatus,
    pub target: &'a Option<String>,
}

impl<'a> fmt::Display for Binary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = if let Some(t) = self.target {
            format!(" [{t}]")
        } else {
            "".to_string()
        };

        write!(f, "{}{target}: {}", self.name, self.status)
    }
}
