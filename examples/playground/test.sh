#/bin/sh

# IMPORTANT run from the workspace root

export CARGO_MANIFEST_DIR="${PWD}/examples/playground/"
export CARGO_PKG_NAME="playground"
export PROFILE=dev
# export PROFILE=release

cargo run --bin=builder-poc -- prebuild --dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}"
cargo build -p playground --target=wasm32-unknown-unknown --profile="${PROFILE}"

cargo run --bin=builder-poc -- postbuild --dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}"

