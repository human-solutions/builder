#![allow(dead_code)]

mod binstall;
mod command;
mod hook;
mod hooks;
mod targets;

use binstall::Binstall;
use toml::{map::Map, Value};

use command::CommandResult;

pub fn run(verbose: bool, no_hooks: bool) -> CommandResult<()> {
    let cargo_install_section = cargo_toml_install_section().unwrap();

    let mut commands = vec![];

    if let Some(targets) = read_targets(&cargo_install_section) {
        commands.push(Some(targets::install(&targets)));
    }

    let hooks_handle = if no_hooks {
        None
    } else {
        hooks::setup(verbose, &cargo_install_section)
    };

    let (binstall, to_install) =
        read_binaries(&cargo_install_section).expect("Failed to read Cargo.toml");

    if let Some(cmd) = binstall.to_command(verbose) {
        cmd.wait(verbose)?;
    }

    commands.extend(to_install.iter().map(|todo| todo.to_command(verbose)));

    for cmd in commands.into_iter().flatten() {
        cmd.wait(verbose)?;
    }
    if let Some(hooks_handle) = hooks_handle {
        hooks_handle.join().unwrap();
    }
    Ok(())
}

fn read_targets(section: &Map<String, Value>) -> Option<Vec<String>> {
    let targets = section.get("targets")?.as_array()?;
    let targets = targets
        .iter()
        .map(|t| t.as_str().map(|s| s.to_owned()))
        .collect::<Option<Vec<_>>>()?;
    Some(targets)
}

fn read_binaries(section: &Map<String, Value>) -> Option<(Binstall, Vec<Binstall>)> {
    let binaries = section.get("binaries")?.as_table()?;
    let mut to_install = vec![];
    let mut binstall = None;
    for (name, entry) in binaries {
        if let Some(entry) = read_binary(name, entry) {
            if name == "cargo-binstall" {
                binstall = Some(entry);
            } else {
                to_install.push(entry);
            }
        } else {
            println!(
                "Failed to read Cargo.toml [workspace.metadata.binstall] entry: {name}: {entry:?}"
            );
        }
    }
    let binstall = binstall.expect(
        "The Cargo.toml [workspace.metadata.binstall] sections has to specify the cargo-binstall",
    );
    Some((binstall, to_install))
}

fn read_binary(name: &str, entry: &Value) -> Option<Binstall> {
    let name = name.to_owned();
    let (version, url) = if entry.as_str().is_some() {
        (entry.as_str().map(|s| s.to_owned()), None)
    } else {
        let version = entry
            .get("version")
            .and_then(|s| s.as_str())
            .map(|s| s.to_owned());
        let url = entry
            .get("github")
            .and_then(|s| s.as_str())
            .map(|s| s.to_owned());
        (version, url)
    };
    Some(Binstall {
        name: name.to_owned(),
        version: version.to_owned(),
        url,
    })
}
fn cargo_toml_install_section() -> Option<Map<String, Value>> {
    let toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let toml: Value = toml::from_str(&toml).unwrap();
    let toml = toml
        .get("workspace")?
        .as_table()?
        .get("metadata")?
        .as_table()?
        .get("install")?
        .as_table();
    toml.map(|t| t.to_owned())
}
