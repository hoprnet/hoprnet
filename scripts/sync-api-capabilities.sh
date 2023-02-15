#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="sync-api-capabilities"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "This script syncs endpoints configured for API capabilities with the existing API endpoints."
  msg
  msg "Usage: $0"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

declare spec_file_path="${mydir}/../packages/hoprd/rest-api-v2-full-spec.json"
declare partial_spec_file_path="${mydir}/../packages/hoprd/rest-api-v2-spec.yaml"
declare caps_file_path="${mydir}/../packages/hoprd/src/api/v2/supported-api-capabilities.json"
declare endpoints

# get all known operation ids and add empty capabilities as the basic configuration
jq '.paths[] | .post, .get, .delete, .put, .head | select(. != null) | .operationId' "${spec_file_path}" |
  jq -s 'map({ (.): {} }) | add' > "${caps_file_path}.base"

# merge with existing configuration
jq -s add "${caps_file_path}.base" "${caps_file_path}" > "${caps_file_path}.merged"

mv "${caps_file_path}.merged" "${caps_file_path}"
rm "${caps_file_path}.base"

# get list of endpoints from newly updated caps file
endpoints="$(jq -r "to_entries | map(.key)" ${caps_file_path})"

# update list in API documentation
cat "${partial_spec_file_path}" | \
  yq -y --argjson endpoints "${endpoints}" \
  ".components.schemas.TokenCapability.properties.endpoint.enum = \$endpoints" > \
  "${partial_spec_file_path}.updated"
mv "${partial_spec_file_path}.updated" "${partial_spec_file_path}"
