#/bin/sh

echo "Install builder"
cargo install --path=crates/builder-poc --profile=dev --offline

echo "Build playground"
touch examples/playground/build.rs
cargo build -p playground --target=wasm32-unknown-unknown -r

builder postbuild --dir=examples/playground --profile=release --package=playground