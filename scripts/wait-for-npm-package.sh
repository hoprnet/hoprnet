#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

declare mydir
declare npm_package

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
