#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <package-name> <package-version> [<wait-interval>]" >&2
  echo >&2
  echo -e "\twhere <wait-interval> is in seconds, default is 5" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
[ -z "${1:-}" ] && { echo "Missing parameter <package-name>" >&2; usage; exit 1; }
[ -z "${2:-}" ] && { echo "Missing parameter <package-version>" >&2; usage; exit 1; }

declare mydir
declare package_name
declare package_version
declare wait_interval

mydir=$(dirname $(readlink -f $0))
package_name="$1"
package_version="$2"
wait_interval="${3:-5}"

# do work

declare npm_package

while : ; do
  npm_package="$(${mydir}/get-npm-package-info.sh ${package_name} ${package_version})"

  # stop if we received a result
  test -n "${npm_package}" && break

  # sleep x seconds before the next run
  sleep ${wait_interval}
done
