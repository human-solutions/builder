use builder_fontforge::{fontforge, Cli};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    fontforge(&args);
}
