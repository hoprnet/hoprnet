#!/bin/bash

target=$1
testcrate_dir="$(pwd)/testcrate"
set -ex

if [ "$1" = "aarch64-apple-darwin" ] ; then
  export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER=echo
fi

cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv
cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv --release

if [ "$1" = "x86_64-unknown-linux-gnu" ] ; then
  cargo test --manifest-path "$testcrate_dir/Cargo.toml" --target $1 -vv --all-features

  # Run a few tests here:
  #
  # * Make sure the packaged crate file isn't bigger than 10MB which is
  #   crate.io's limit.
  # * Make sure that the package crate itself works.
  #
  # A lot of OpenSSL's source code is excluded on crates.io because it makes the
  # crate file much too large, so the test here should inform us if we're
  # missing anything actually required to build OpenSSL.
  rm -rf target/ci
  cargo package --allow-dirty --target-dir target/ci
  crate=`ls target/ci/package/*.crate`
  filesize=$(stat -c%s "$crate")
  echo "tarball is $filesize bytes"
  if (( filesize > 10000000 )); then
    echo "file size too big"
    exit 1
  fi
  rm "$crate"
  cd target/ci/package/openssl-src-*
  cp -r "$testcrate_dir" .
  cargo test --manifest-path "testcrate/Cargo.toml" --target $1 -vv
fi
