use anyhow::Result;

use crate::{input::Input, BuilderArgs};

pub struct Builder;

impl Builder {
    pub fn run(args: BuilderArgs) -> Result<()> {
        // parse cargo.toml
        // parse builder.toml
        // check if needed binaries are installed
        // gather all the output.yaml generated
        // read the envs
        // create input.yaml

        let input = Input::gather_all(&args.dir)?;

        // for (key, val) in input.configs.0 {
        //     println!("{key:?} -> {val}");
        // }
        // for (key, val) in input.binaries.0 {
        //     println!("{key:?} -> {val:?}");
        // }

        input.save_file()?;

        Ok(())
    }
}
