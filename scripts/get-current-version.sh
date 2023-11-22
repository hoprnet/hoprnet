#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <version_type[semver|docker]>"
  echo ""
  echo "$0 semver"
  echo "$0 docker"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

version_type=${1} # Can be any of these values Build | ReleaseCandidate | Patch | Minor | Major
current_version=$(sed -n "s/^version = //p" ${mydir}/../packages/hoprd/crates/hopr-lib/Cargo.toml | tr -d '"')

if [ "${version_type}" == "docker" ] 
then
  echo ${current_version}
else
  echo $(echo ${current_version} | sed 's/+/-/')
fi
