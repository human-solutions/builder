use clap::Parser;

fn main() {
    let cmd = Cli::parse();
    cmd.run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Cli {
    Sass(builder_sass::Cli),
    Localized(builder_localized::Cli),
    Uniffi(builder_uniffi::Cli),
    Fontforge(builder_fontforge::Cli),
}

impl Cli {
    fn run(&self) {
        match self {
            Self::Sass(args) => builder_sass::run(args),
            Self::Localized(args) => builder_localized::run(args),
            Self::Uniffi(args) => builder_uniffi::run(args),
            Self::Fontforge(args) => builder_fontforge::fontforge(args),
        }
    }
}
