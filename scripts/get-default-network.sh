#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-default-network"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

declare key_to_extract=".value.network"

if [[ "${1:-}" = "--release" ]] ; then
  log "Getting the release id"
  key_to_extract=".key"
else
  log "Getting the network id"
fi

log "get default network id"
declare branch=$(git rev-parse --abbrev-ref HEAD)

declare network
for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    network=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\" and .value.default==true) | ${key_to_extract}")
    # if no default is set we take the first entry
    if [ -z "${network}" ]; then
      network=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | ${key_to_extract}" | sed q)
    fi
    break
  fi
done

: "${network:?"Could not read value for default network id"}"

log "found default network: ${network}"

echo "${network}"
