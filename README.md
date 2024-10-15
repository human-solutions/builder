## Releasing a new version

Releases are pre-compiled with `cargo dist` for various platforms and uploaded to github releases.
These can be used by `cargo binstall` to install the binary.

1. Update the version in `Cargo.toml`.
2. Add a git tag with the version number. Ex: `git tag v0.0.1 -m"Version 0.0.1: message"`.
3. Push the tag to the repository. Ex: `git push --tags`.
