#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="notify-matrix"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [<cluster_tag>]"
  msg
  msg "The following environment variables are used to perform the request:"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "MATRIX_ROOM\t\tdefines the room the message is sent to"
  msg "MATRIX_ACCESS_TOKEN\t\ttoken used for authenticating against Matrix"
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes"
  msg
  msg "Optional environment variables"
  msg "------------------------------"
  msg
  msg "MATRIX_SERVER, defaults to 'https://matrix.org'"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# do work
which curl > /dev/null || { msg "Required binary 'curl' not found in PATH"; exit 1; }
which jq > /dev/null || { msg "Required binary 'jq' not found in PATH"; exit 1; }

: "${HOPRD_API_TOKEN:?"env var missing"}"
: "${MATRIX_ROOM:?"env var missing"}"
: "${MATRIX_ACCESS_TOKEN:?"env var missing"}"

declare cluster_tag=${1:-} # optional cluster tag
declare room="${MATRIX_ROOM}"
declare branch
branch=$(git rev-parse --abbrev-ref HEAD)

_jq() {
  echo "$1" | base64 --decode | jq -r "$2"
}

for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    for row in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | @base64"); do
      declare release_id version_major version_minor cluster_name version_maj_min msg now

      release_id=$(_jq "${row}" ".key")
      version_major=$(_jq "${row}" ".value.version_major")
      version_minor=$(_jq "${row}" ".value.version_minor")

      if [[ "${branch}" =~ staging/.* ]]; then
        # Prepend "staging-" tag prefix, if this is a staging branch
        cluster_tag="-staging${cluster_tag}"
      fi

      cluster_name="${release_id}${cluster_tag}"

      if [ "${version_major}" != "null" ] && [ "${version_minor}" != "null" ]; then
        version_maj_min="$version_major.$version_minor"
        cluster_name="${cluster_name}-${version_maj_min//./-}"
      fi

      msg="$("${mydir}/get-gcloud-instances-accounts-info.sh" "${cluster_name}")"

      # create a table out of the info
      msg="<table>$(echo "${msg}" | sed 's/\(.*\)$/<tr><td>\1<\/td><\/tr>/g' | sed 's/\s\{2,\}/<\/td><td>/g' | tr -d '\n')</table>"

      # prepend with title
      now="$(date -u)"
      msg="<h5>Cluster updated: ${cluster_name}</h5><span>Timestamp: ${now}</span>${msg}"

      "${mydir}"/notify-matrix.sh "${room}" "${msg}"
    done
  fi
done


