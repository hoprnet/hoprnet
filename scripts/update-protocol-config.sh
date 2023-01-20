#!/usr/bin/env bash

# # prevent sourcing of this script, only allow execution
# $(return >/dev/null 2>&1)
# test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="update-protocol-config"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-e|--environment_id <environment_id>]"
  msg
  msg "If <environment_id> is not given, use anvil-localhost."
  msg "<environment_id> is used to determine for which environments the contracts shall be deployed."
  msg
}

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -e|--environment_id)
      environment_id="${2}"
      : ${environment_id?"parameter <environment_id> must not be empty"}
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

declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"

update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "${environment_id}" "${environment_id}"
