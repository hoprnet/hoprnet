#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-package-version"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [<pkg>]"
  msg
  msg "\t<pkg> must be one of packages under packages/, defaults to 'hoprd'"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

declare pkg pkg_path version

pkg=${1:-hoprd}
pkg_path="${mydir}/../packages/${pkg}/package.json"

log "Get package version for ${pkg} from ${pkg_path}"

# get full version info from package description
version=$(jq -r '.version' "${pkg_path}")

echo "${version}"
