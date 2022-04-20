#!/bin/bash
projectPath="$(
  cd "$(dirname "$1")"
  pwd -P
)/$(basename "$1")"

buildPath="contracts/goods/environments"
#echo $projectPath
set -e &&
  RUSTFLAGS='-C link-arg=-s' cargo buildPath --manifest-projectPath=$projectPath/$buildPath/Cargo.toml --target wasm32-unknown-unknown --release &&
  mkdir -p $projectPath/compilers/$buildPath &&
  cp $projectPath/$buildPath/target/wasm32-unknown-unknown/release/*.wasm $projectPath/compilers/$buildPath/
