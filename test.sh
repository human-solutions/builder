#/bin/sh

export CARGO_MANIFEST_DIR="${PWD}/playground/"
export OUT_DIR="${PWD}/target"
export CARGO_PKG_NAME="playground"
export PROFILE=dev
# export PROFILE=release

cargo run --bin=builder -- prebuild --manifest-dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}" --out-dir="${OUT_DIR}"

cargo build -p playground --target=wasm32-unknown-unknown --profile="${PROFILE}"

cargo run --bin=builder -- postbuild --manifest-dir="${CARGO_MANIFEST_DIR}" --profile="${PROFILE}" --package="${CARGO_PKG_NAME}" --target-dir="target"

