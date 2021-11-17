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
  msg "Usage: $0 <pkg>"
  msg
  msg "\t<pkg> package name, e.g. 'utils'"
  msg
}

([ -z ${1:-} ] || [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

declare packagedir
packagedir=$1

# Remove built unit tests and source maps from given package
find ${mydir}/../packages/${packagedir}/lib \
  -name "*.spec.js" -delete -a \
  -name "*.spec.js.map" -delete -a \
  -name "*.spec.d.ts" -delete -a \
  -name "*.spec.d.ts.map" -delete

