use super::hook::{Hook, HookStage};
use colorful::Colorful;
use std::{
    fs,
    str::FromStr,
    thread::{self, JoinHandle},
};
use toml::{map::Map, Value};

pub fn setup(verbose: bool, cargo_install_section: &Map<String, Value>) -> Option<JoinHandle<()>> {
    let githooks = cargo_install_section.get("githooks")?.as_table()?;

    let mut hooks = vec![];

    for (name, entry) in githooks {
        let stage = HookStage::from_str(name).expect("Invalid hook stage");
        let script = entry.as_str().expect("Invalid hook script").to_string();
        hooks.push(Hook { stage, script });
    }

    Some(thread::spawn(move || {
        if hooks.is_empty() {
            verbose.then(|| warn("hook", "No custom hooks found"));
            return;
        }

        for hook in &hooks {
            let create = fs::metadata(hook.git_path()).is_err();

            verbose.then(|| println!("Setting up {} script ...", hook.stage));

            if let Err(e) = hook.install(create) {
                warn("hook", &e);
                continue;
            }

            verbose.then(|| success("hook", &format!("{} script installed", hook.stage)));
        }
    }))
}

pub fn warn(prefix: &str, msg: &str) {
    println!("{} {msg}", prefix.yellow());
}

pub fn success(prefix: &str, msg: &str) {
    if !msg.is_empty() {
        println!("{} {msg}", prefix.green());
    }
}
