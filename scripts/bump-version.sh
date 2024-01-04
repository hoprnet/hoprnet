#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <new_version>"
  echo ""
  echo "\t<new_version> New version to be applied"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

new_version="${1}"

# Update hopr-lib Rust manifest
sed -i'.original' 's/^version = ".*"$/version = "'${new_version}'"/' ${mydir}/../packages/hoprd/crates/hopr-lib/Cargo.toml
rm ${mydir}/../packages/hoprd/crates/hopr-lib/Cargo.toml.original
