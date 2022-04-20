#!/bin/bash
echo "*- Start compile environments -*"

projectPath="$(
  cd "$(dirname "$1")"
  pwd -P
)/$(basename "$1")"

buildPath="contracts/goods/environments"
#echo $projectPath
echo -ne '>>>                       [20%]\r'
set -e
RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=$projectPath/$buildPath/Cargo.toml --target wasm32-unknown-unknown --release
echo -ne '>>>>>>>                   [40%]\r'
mkdir -p $projectPath/compilers/$buildPath
echo -ne '>>>>>>>>>>>>>>            [60%]\r'
cp $projectPath/$buildPath/target/wasm32-unknown-unknown/release/*.wasm $projectPath/compilers/$buildPath/
echo -ne '>>>>>>>>>>>>>>>>>>>>>>>   [80%]\r'
echo "--> Compile Successfully -"
find $projectPath/compilers/$buildPath
echo -ne '>>>>>>>>>>>>>>>>>>>>>>>>>>[100%]\r'
echo -ne '\n'
echo "*- End compile environments -*"
