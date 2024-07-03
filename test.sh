#/bin/sh

export CARGO_MANIFEST_DIR="${PWD}/playground/"
export CARGO_PKG_NAME="playground"
export PROFILE=dev
# export PROFILE=release

cargo run --bin=builder -- prebuild --dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}"
cargo build -p playground --target=wasm32-unknown-unknown --profile="${PROFILE}"

cargo run --bin=builder -- postbuild --dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}"

