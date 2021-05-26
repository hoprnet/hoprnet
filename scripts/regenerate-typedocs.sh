#!/usr/bin/env bash


# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters

declare mydir

mydir=$(dirname $(readlink -f $0))

# do work

# remove previously generated docs to ensure renamed/removed modules are not kept in the docs
rm -rf "${mydir}/../packages/*/docs"

cd "${mydir}"
yarn build
yarn docs:generate
