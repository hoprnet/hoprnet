#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC2034
declare HOPR_LOG_ID="bump-version"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <suffix>"
  msg
  msg "\t<suffix> Suffix to add to the current version"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
declare version_suffix

# Get the version suffix from input parameters
version_suffix="${1:-}"

# define packages for versioning
# does not include ethereum, which isn't a real package anymore, just a folder
declare -a versioned_packages=( utils connect core-ethereum core real hoprd )

declare current_version
current_version="$("${mydir}/get-package-version.sh")"

declare new_version
new_version="${current_version}-pr-${version_suffix}"

# create new version in each package
for package in "${versioned_packages[@]}"; do
  log "creating new version ${new_version} in package ${package}"
  cd "${mydir}/../packages/${package}"
  jq ".version = \"${new_version}\"" package.json > package.json.new
  mv package.json.new package.json
  cd "${mydir}/.."
done
