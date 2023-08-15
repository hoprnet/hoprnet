#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-npm-package-info"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [<pkg>] [<pkg_version>]"
  msg
  msg "\t<pkg> must be one of packages under packages/, defaults to 'hoprd'"
  msg "\t<pkg_version> must be a valid semver version which shall be checked, defaults to the latest tag"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare pkg pkg_vsn pkg_name pkg_path

pkg=${1:-hoprd}
pkg_vsn=${2:-$("${mydir}/get-package-version.sh" "${pkg}")}
pkg_path="${mydir}/../packages/${pkg}/package.json"
pkg_name=$(jq -r '.name' "${pkg_path}")

log "Get npm package info for ${pkg_name}@${pkg_vsn}"

# handle errors in the npm execution gracefully
npm view "${pkg_name}@${pkg_vsn}" --json || echo ""
