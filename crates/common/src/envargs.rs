use camino_fs::Utf8PathBuf;

#[derive(Debug, Clone)]
pub struct CargoEnv {
    pub dir: Utf8PathBuf,
    pub profile: String,
    pub package: String,
    pub target: String,
}

impl CargoEnv {
    pub fn from_env() -> Self {
        Self {
            dir: Utf8PathBuf::from(env("CARGO_MANIFEST_DIR")),
            profile: env("PROFILE"),
            package: env("CARGO_PKG_NAME"),
            target: env("TARGET"),
        }
    }
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap().to_string()
}
