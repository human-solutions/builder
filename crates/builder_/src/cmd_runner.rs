use std::process::{Child, Command, ExitCode, Output, Stdio};

pub type CmdResult<T> = Result<T, ExitCode>;

pub struct CmdRunner {
    child: Child,
}

impl CmdRunner {
    pub fn run(prog: &str, args: &[&str]) -> Self {
        let child = Command::new(prog)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        Self { child }
    }

    pub fn output(self) -> CmdResult<Output> {
        self.child.wait_with_output().map_err(|_| ExitCode::FAILURE)
    }
}
