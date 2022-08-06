#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
source "${mydir}/utils.sh"

: ${1:?"No <endpoint> is set, use default value localhost:3001"}

# $1 = optional: endpoint, defaults to http://localhost:3001
declare endpoint="${1:-localhost:3001}"
declare url="${endpoint}/api/v2/account/addresses"
declare cmd="$(get_authenticated_curl_cmd ${url})"

try_cmd "${cmd}" 30 5 | jq -r ".hopr"