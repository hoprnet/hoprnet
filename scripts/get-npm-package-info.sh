#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare pkg pkg_vsn pkg_name mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-npm-package-info"
source "${mydir}/utils.sh"

pkg=${HOPR_PACKAGE:-hoprd}
pkg_vsn=${HOPR_PACKAGE_VERSION:-$(HOPR_PACKAGE=${pkg} ${mydir}/get-package-version.sh)}
pkg_name="@hoprnet/${pkg}"

log "Get npm package info for ${pkg_name}@${pkg_vsn}"
npm view "${pkg_name}@${pkg_vsn}" --json
