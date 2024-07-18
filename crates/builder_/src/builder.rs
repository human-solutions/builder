use anyhow::Result;

use crate::{types::Input, BuilderArgs};

pub fn run(args: BuilderArgs) -> Result<()> {
    let input = Input::gather(args)?;

    input.save_file()?;

    Ok(())
}
