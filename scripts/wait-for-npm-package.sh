#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir npm_package

test -z "${HOPR_PACKAGE:-}" && (echo "Missing environment variable HOPR_PACKAGE"; exit 1)
test -z "${HOPR_PACKAGE_VERSION:-}" && (echo "Missing environment variable HOPR_PACKAGE_VERSION"; exit 1)

mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

while : ; do
  npm_package=$(${mydir}/get-npm-package-info.sh)

  # stop if we received a result
  test -n "${npm_package}" && break

  # sleep x seconds before the next run
  sleep ${HOPR_WAIT_INTERVAL:-5}
done
