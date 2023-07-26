#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="deploy-smart-contracts"
# shellcheck disable=SC1090
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
      : "${branch?"parameter <branch> must not be empty"}"
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

declare release_config="${mydir}/../packages/hoprd/releases.json"
declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"

for git_ref in $(cat "${release_config}" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    for network in $(cat "${release_config}" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .value.network"); do
      declare chain=$(cat "${protocol_config}" | jq -r ".networks.\"${network}\".chain")
      declare environment_type=$(cat "${deployments_summary}" | jq -r ".networks.\"${network}\".environment_type")

      log "deploying for network ${network} on chain ${chain} of type ${environment_type}"

      make -C "${mydir}/../packages/ethereum/contracts/" anvil-deploy-contracts network="${network}" environment-type="${environment_type}"

      # update the deployed files in protocol-config
      update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "${network}" "${network}"
    done
  fi
done
