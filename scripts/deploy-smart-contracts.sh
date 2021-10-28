#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir branch
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
branch=$(git rev-parse --abbrev-ref HEAD)

cd "${mydir}/../"

for environment_id in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r ".[] | select(.git_ref==\"refs/heads/${branch}\") | .id"); do
  local network_id=$(cat "${mydir}/../packages/core/protocol-config.json" | jq -r ".environments.\"${environment_id}\".network_id")

  # deploy smart contracts
  yarn workspace @hoprnet/hopr-ethereum hardhat deploy --network "${network_id}"
done
