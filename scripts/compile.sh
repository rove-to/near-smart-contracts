#!/bin/bash

rust_files=$(find ./*/** -type f -name "lib.rs")
for i in $rust_files; do
  dir_path=$(dirname "$i")
  parentdir="$(dirname "$dir_path")"
  echo $parentdir

  echo "\n\n\n\n\n ************************************"
  rm -rf $parentdir/target
  echo "rm -rf $parentdir/target"
  sh $parentdir/build.sh
  echo "run build $parentdir/build.sh"
done
