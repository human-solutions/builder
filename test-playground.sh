#/bin/sh

echo "Install builder"
cargo install --path=cmd --profile=dev --offline

echo "Build playground"
touch playground/build.rs
cargo build -p playground
