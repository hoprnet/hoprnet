#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && (echo "This script should only be executed."; exit 1)

usage() {
  echo "Usage: $0 <package-name> <package-version>" >&2
  echo
  exit 0
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && usage

# verify parameters
[ -z "${1:-}" ] && (echo "Missing first parameter <package-name>"; usage; exit 1)
[ -z "${2:-}" ] && (echo "Missing second parameter <package-version>"; usage; exit 1)

# do work
declare mydir
declare npm_package

mydir=$(dirname $(readlink -f $0))

while : ; do
  npm_package=$(${mydir}/get-npm-package-info.sh)

  # stop if we received a result
  test -n "${npm_package}" && break

  # sleep x seconds before the next run
  sleep ${HOPR_WAIT_INTERVAL:-5}
done
