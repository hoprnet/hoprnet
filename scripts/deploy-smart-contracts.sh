#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <network>" >&2
  echo >&2
  echo -e "\texpects the environment variables PRIVATE_KEY and QUIKNODE_KEY to be set" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
[ -z "${1:-}" ] && { echo "Missing parameter <network>" >&2; usage; exit 1; }
[ -z "${PRIVATE_KEY:-}" ] && { echo "Missing environment variable PRIVATE_KEY" >&2; exit 1; }
[ -z "${QUIKNODE_KEY:-}" ] && { echo "Missing environment variable QUIKNODE_KEY" >&2; exit 1; }

declare mydir
declare network

mydir=$(dirname $(readlink -f $0))
network="$1"

# do work

# go to ethereum package
cd "${mydir}/../packages/ethereum"

# deploy smart contracts
yarn deploy --network "${network}"
