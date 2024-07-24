use super::command::Command;

pub fn install(targets: &[String]) -> Command {
    let mut arguments = vec!["target", "add"];
    arguments.extend(targets.iter().map(String::as_str));
    Command::run("rustup", &arguments)
}
