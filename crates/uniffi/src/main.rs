use builder_uniffi::{run, Cli};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    run(&args);
}
