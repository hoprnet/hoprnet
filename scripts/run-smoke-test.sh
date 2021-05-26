#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <endpoint>" >&2
  echo >&2
  echo -e "\twhere <endpoint> is a hoprd endpoint, default is http://localhost:3001" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare endpoint

endpoint="${1:-http://localhost:3001}"

# Smoke test a running node

# $1 - command
# $2 - admin host
cmd() {
  curl --data "$1" "$2/api/v1/command"
}

# Info
cmd 'version' "${endpoint}"
cmd 'settings' "${endpoint}"
cmd 'info' "${endpoint}"
cmd 'balance' "${endpoint}"
cmd 'address' "${endpoint}"
cmd 'channels' "${endpoint}"
cmd 'peers' "${endpoint}"
