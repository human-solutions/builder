use std::{
    fmt::Display,
    process::{self, ExitCode, ExitStatus, Stdio},
};

pub type CommandResult<T> = Result<T, ExitCode>;

pub struct Command {
    child: process::Child,
}
pub enum UpdateAction {
    Installing,
    Updating,
}

impl Display for UpdateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateAction::Installing => write!(f, "installing"),
            UpdateAction::Updating => write!(f, "updating"),
        }
    }
}

impl Command {
    pub fn cargo_install(prog: &str, version: &Option<String>, verbose: bool) -> Option<Self> {
        Self::check(prog, version).map(|reason| {
            let prog_version = if let Some(v) = version {
                println!("{reason} {prog}@{v}");
                format!("{prog}@{v}")
            } else {
                println!("{reason} {prog}@latest");
                prog.to_string()
            };

            let args = if verbose {
                vec!["install", "-v", &prog_version, "--locked"]
            } else {
                vec!["install", &prog_version, "--locked"]
            };

            Self::run("cargo", &args)
        })
    }

    pub fn binstall(prog: &str, version: &Option<String>, verbose: bool) -> Option<Self> {
        Self::check(prog, version).map(|reason| {
            let prog_version = if let Some(v) = version {
                println!("{reason} {prog}@{v}");
                format!("{prog}@{v}")
            } else {
                println!("{reason} {prog}@latest");
                prog.to_string()
            };

            let args = if verbose {
                vec!["-v", &prog_version, "-y"]
            } else {
                vec![&prog_version, "-y"]
            };

            Self::run("cargo-binstall", &args)
        })
    }

    pub fn binstall_git(
        prog: &str,
        version: &Option<String>,
        url: &str,
        verbose: bool,
    ) -> Option<Self> {
        Self::check(prog, version).map(|reason| {
            let prog_version = if let Some(v) = version {
                println!("{reason} {prog}@{v}");
                format!("{prog}@{v}")
            } else {
                println!("{reason} {prog}@latest");
                prog.to_string()
            };

            let args = if verbose {
                vec!["-v", &prog_version, "-y", "--git", url]
            } else {
                vec![&prog_version, "-y", "--git", url]
            };

            Self::run("cargo-binstall", &args)
        })
    }

    pub fn run(program: &'static str, arguments: &[&str]) -> Self {
        let child = process::Command::new(program)
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        Self { child }
    }

    pub fn check(cmd: &str, version: &Option<String>) -> Option<UpdateAction> {
        let command_result = if cmd == "cargo-ndk" {
            process::Command::new("cargo")
                .args(["ndk", "-v"])
                .output()
                .map(CommandOutput::new)
        } else {
            process::Command::new(cmd)
                .arg("-V")
                .output()
                .map(CommandOutput::new)
        };

        if let Some(version) = version {
            match command_result {
                Err(_) => Some(UpdateAction::Installing),
                Ok(output) => {
                    if output.stdout.contains(version) {
                        None
                    } else {
                        Some(UpdateAction::Updating)
                    }
                }
            }
        } else if command_result.is_err() {
            Some(UpdateAction::Installing)
        } else {
            None
        }
    }

    pub fn wait(self, verbose: bool) -> CommandResult<()> {
        let Ok(output) = self.child.wait_with_output() else {
            return Err(ExitCode::FAILURE);
        };
        print_output(output, verbose)
    }
}

struct CommandOutput {
    stdout: String,
    _stderr: String,
    _success: bool,
}

impl CommandOutput {
    fn new(output: process::Output) -> Self {
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        let success = output.status.success();
        Self {
            stdout,
            _stderr: stderr,
            _success: success,
        }
    }
}
fn print_output(output: process::Output, verbose: bool) -> CommandResult<()> {
    let status = output.status;
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if verbose {
        println!("{stdout}");
        if !status.success() {
            println!("{stderr}");
        }
    }

    status.to_result()
}

pub trait ExitConvert {
    fn to_code(&self) -> ExitCode;
    fn to_result(&self) -> CommandResult<()>;
}

impl ExitConvert for ExitStatus {
    fn to_code(&self) -> ExitCode {
        let code = self
            .code()
            .map(|code| code.try_into().unwrap_or(1))
            .unwrap_or(1);
        ExitCode::from(code)
    }

    fn to_result(&self) -> CommandResult<()> {
        if self.success() {
            Ok(())
        } else {
            Err(self.to_code())
        }
    }
}
