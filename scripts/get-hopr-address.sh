#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

# $1 = optional: apitoken, defaults to ""
declare apitoken="${1:-}"
# $2 = optional: endpoint, defaults to http://localhost:3001
declare endpoint="${2:-localhost:3001}"

if [[ -z "${apitoken}" ]]; then
    msg "No <apitoken> is set"
    exit 1
fi
if [[ -z "${endpoint}" ]]; then
    msg "No <endpoint> is set, use default value localhost:3001"
fi

declare url="${apitoken}@${endpoint}/api/v3/account/addresses"

declare cmd="$(get_authenticated_curl_cmd "${url}")"

try_cmd "${cmd}" 30 5 | jq -r ".hopr"