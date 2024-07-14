use anyhow::Result;

use crate::BuilderArgs;

// Input :
// - envs
// - plugins : Vec<Plugin>
// - binaries
// - githooks

// [phase.assembly.target.profile.plugin.action]
// output

// Output:
// key = value
// key1 = value1

// Plugin :
// - name
// - prebuild : Vec<Action>
// - postbuild : Vec<Action>

// Action :
// - name
// - assembly
// - target
// - profile
// - output

// Binary :
// - name
// - triple: Vec<Triple>

// Triple :
// - host
// - installer : Vec<Installer>

// Installer :
// - type
// - args
// - version
// - version-arg
// - watch

pub fn run(args: BuilderArgs) -> Result<()> {
    // parse cargo.toml
    // parse builder.toml
    // check if needed binaries are installed
    // gather all the output.yaml generated
    // read the envs
    // create input.yaml

    // let input = Input::gather_all(&args.dir)?;

    // for (key, val) in input.configs.0 {
    //     println!("{key:?} -> {val}");
    // }
    // for (key, val) in input.binaries.0 {
    //     println!("{key:?} -> {val:?}");
    // }

    // input.save_file()?;

    Ok(())
}
