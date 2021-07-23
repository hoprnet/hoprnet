#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare pkg pkg_path mydir version
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-package-version"
source "${mydir}/utils.sh"

pkg=${HOPR_PACKAGE:-hoprd}
pkg_path="${mydir}/../packages/${pkg}/package.json"

log "Get package version for ${pkg} from ${pkg_path}"

# get full version info from package description
version=$(node -p -e "require('${pkg_path}').version")

# returns the maj-min version if requested, otherwise the full version
if [ -n "${HOPR_VERSION_MAJMIN:-}" ]; then
  echo "${version}" | sed 's/\(\.[0-9]*\-alpha\)*\.[0-9]*$//'
else
  echo "${version}"
fi
