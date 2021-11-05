#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

declare HOPR_LOG_ID="remove tests"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [<pkg_dir>]"
  msg
  msg "\t<pkg_dir> package directory, e.g. 'packages/utils'"
  msg
}

([ -z ${1:-} ] || [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

declare packagedir
packagedir=$1

# Remove built unit tests and source maps from given directory
rm -Rf ${mydir}/../${packagedir}/lib/**/*.spec.js ${mydir}/../${packagedir}/lib/*.spec.js
rm -Rf ${mydir}/../${packagedir}/lib/**/*.spec.js.map ${mydir}/../${packagedir}/lib/*.spec.js.map
rm -Rf ${mydir}/../${packagedir}/lib/**/*.spec.d.ts ${mydir}/../${packagedir}/lib/*.spec.d.ts
rm -Rf ${mydir}/../${packagedir}/lib/**/*.spec.d.ts.map ${mydir}/../${packagedir}/lib/*.spec.d.ts.map
