#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

rm -Rf ${mydir}/lib/**/*.spec.js
rm -Rf ${mydir}/lib/**/*.spec.js.map
rm -Rf ${mydir}/lib/**/*.spec.d.ts
rm -Rf ${mydir}/lib/**/*.spec.d.ts.map
