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

declare release_config="${mydir}/../packages/hoprd/releases.json"
declare protocol_config="${mydir}/../packages/core/protocol-config.json"

for git_ref in $(cat "${release_config}" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    for environment_id in $(cat "${release_config}" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .value.environment_id"); do
      declare network_id=$(cat "${protocol_config}" | jq -r ".environments.\"${environment_id}\".network_id")

      log "deploying for environment ${environment_id} on network ${network_id}"

      # We need to pass the --write parameter due to hardhat-deploy expecting that
      # to be set in addition to the hardhat config saveDeployments.
      # See:
      # https://github.com/wighawag/hardhat-deploy/blob/8c76e7f942010d09b3607650042007f935401633/src/DeploymentsManager.ts#L503
      HOPR_ENVIRONMENT_ID="${environment_id}" yarn workspace @hoprnet/hopr-ethereum \
        hardhat deploy --network "${network_id}" --write true

      log "updating contract addresses in protocol configuration"

      declare token_contract_address channels_contract_address deployments_path

      deployments_path="${mydir}/../packages/ethereum/deployments/${environment_id}/${network_id}"
      token_contract_address="$(cat "${deployments_path}/HoprToken.json" | jq -r ".address")"
      channels_contract_address="$(cat "${deployments_path}/HoprChannels.json" | jq -r ".address")"

      cat "${protocol_config}" | jq ".environments.\"${environment_id}\".token_contract_address = \"${token_contract_address}\"" > "${protocol_config}.new"
      mv "${protocol_config}.new" "${protocol_config}"

      cat "${protocol_config}" | jq ".environments.\"${environment_id}\".channels_contract_address = \"${channels_contract_address}\"" > "${protocol_config}.new"
      mv "${protocol_config}.new" "${protocol_config}"

    done
  fi
done
