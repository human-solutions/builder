#/bin/sh

export BUILDER_MANIFEST_DIR="${PWD}/playground/"
export BUILDER_OUT_DIR="${PWD}/target/debug/build/playground-6b78a4df1c8142e7/out"
export BUILDER_PKG_NAME="playground"
export BUILDER_PROFILE=debug
export BUILDER_PROFILE=release

cargo run --bin=builder -- prebuild