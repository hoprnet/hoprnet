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

# define packages for versioning
# does not include ethereum, which isn't a real package anymore, just a folder
declare -a versioned_packages=( utils core-ethereum core real hoprd )

# create new version in each package
for package in "${versioned_packages[@]}"; do
  echo "Updating package ${package} to version ${new_version}"
  cd "${mydir}/../packages/${package}"
  jq ".version = \"${new_version}\"" package.json > package.json.new
  mv package.json.new package.json
  cd "${mydir}/.."
done

# Update hopr-lib Rust manifest
sed -i'.original' 's/^version = ".*"$/version = "'${new_version}'"/' ${mydir}/../packages/hoprd/crates/hopr-lib/Cargo.toml
rm ${mydir}/../packages/hoprd/crates/hopr-lib/Cargo.toml.original
