#!/bin/bash
rm -rf ./compilers

rust_files=$(find ./*/** -type f -name "lib.rs")
for i in $rust_files; do
  dir_path=$(dirname "$i")
  parentdir="$(dirname "$dir_path")"
  rm -rf $parentdir/target
  echo "rm -rf $parentdir/target"
done
