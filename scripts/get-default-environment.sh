#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="get-npm-package-info"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0"
  msg
  msg "\t<pkg> must be one of packages under packages/, defaults to 'hoprd'"
  msg "\t<pkg_version> must be a valid semver version which shall be checked, defaults to the latest tag"
  msg
}


log "get default environment id"
declare branch=$(git rev-parse --abbrev-ref HEAD)

declare environment_id
for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    environment_id=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\" and .value.default==true) | .key")
    # if no default is set we take the first entry
    if [ -z "${environment_id}" ]; then
      environment_id=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .key" | sed q)
    fi
    break
  fi
done

: ${environment_id:?"Could not read value for default environment id"}

log "found default environment: ${environment_id}"

echo ${environment_id}