use super::command::Command;

#[derive(Debug)]
pub struct Binstall {
    pub name: String,
    pub version: Option<String>,
    pub url: Option<String>,
}

impl Binstall {
    pub fn to_command(&self, verbose: bool) -> Option<Command> {
        if self.name == "cargo-binstall" {
            Command::cargo_install("cargo-binstall", &self.version, verbose)
        } else if let Some(url) = &self.url {
            let url = format!("https://github.com/{url}");
            Command::binstall_git(&self.name, &self.version, &url, verbose)
        } else {
            Command::binstall(&self.name, &self.version, verbose)
        }
    }
}
