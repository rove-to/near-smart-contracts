#!/bin/bash
path="$(
  cd "$(dirname "$1")"
  pwd -P
)/$(basename "$1")"
build="contracts/goods/environments"
#echo $path
set -e &&
  RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=$path/contracts/goods/environments/Cargo.toml --target wasm32-unknown-unknown --release &&
  mkdir -p $path/compilers/$build &&
  cp $path/$build/target/wasm32-unknown-unknown/release/*.wasm $path/compilers/$build/
