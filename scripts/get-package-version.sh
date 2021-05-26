#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <package-name>" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
[ -z "${1:-}" ] && { echo "Missing parameter <package-name>" >&2; usage; exit 1; }

declare mydir
declare package_name
declare full_version

mydir=$(dirname $(readlink -f $0))
package_name="$1"

# get full version info from package description
full_version=$(node -p -e "require('${mydir}/../packages/${package_name}/package.json').version")

# returns the maj-min version if requested, otherwise the full version
if [ -n "${HOPR_VERSION_MAJMIN:-}" ]; then
  echo "${full_version}" | sed 's/\(\.[0-9]*\-alpha\)*\.[0-9]*$//'
else
  echo "${full_version}"
fi
