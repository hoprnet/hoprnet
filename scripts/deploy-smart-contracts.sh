#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="deploy-smart-contracts"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-b|--branch <branch>]"
  msg
  msg "If <branch> is not given, its inferred by the local Git state."
  msg "<branch> is used to determine for which environments the contracts shall be deployed."
  msg
}

declare branch
branch="$(git rev-parse --abbrev-ref HEAD)"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -b|--branch)
      branch="${2}"
      : ${branch?"parameter <branch> must not be empty"}
      shift 2
      ;;
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done


cd "${mydir}/../"

for environment_id in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r ".[] | select(.git_ref==\"refs/heads/${branch}\") | .id"); do
  declare network_id=$(cat "${mydir}/../packages/core/protocol-config.json" | jq -r ".environments.\"${environment_id}\".network_id")

  log "deploying for environment ${environment_id} on network ${network_id}"

  HOPR_ENVIRONMENT_ID="${environment_id}" yarn workspace @hoprnet/hopr-ethereum hardhat deploy --network "${network_id}"
done
