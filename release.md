## Releasing a new version

Releases are pre-compiled with `cargo dist` for various platforms and uploaded to github releases.
These can be used by `cargo binstall` to install the binary.

1. Update the version in `Cargo.toml`.
2. Add a git tag with the version number. Ex: `git tag v0.0.1 -m"Version 0.0.1: message"`.
3. Push the tag to the repository. Ex: `git push --tags`.

## Features

- **Checksum prefixing**: a checksum can be prefixed to file names in order to guarantee
  that an outdated version is never used. This requires the links to be updated in the
  html as well.
  There are two types of checksums used:
  - The checksum of the file(s).
  - (coming) The date-time of the start of the build checksummed. This is used when the file checksum
    cannot be used, typically for the WASM that generate the html that needs to include
    a link to the WASM file.
- **Localization** An asset can be localized.
- **FontForge building** If fontforge is installed, a font can be generated from a fontforge source file.
- **Style build** compiling SASS to CSS, and in release mode, vendor prefixing, minifying and
  other CSS processing. See lightningcss.
- **JS Minifying**. (coming) Only for release builds.
- **Compression**. Pre-compressing the files in brotli and gzip formats. Only release builds.
- **Packaging**. Moving the file to the correct folder in the `target/<name of my package>` folder.
- **Embedding**. The files can be included in the binary with [rust-embed](https://crates.io/crates/rust-embed).
