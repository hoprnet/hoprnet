#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-gcloud-test"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"

usage() {
  msg
  msg "Usage: $0 [<docker_image>]"
  msg
  msg "\twhere <docker_image> uses latest as default"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare docker_image rpc_endpoint rpc_endpoint_ip
docker_image=${1:-latest}

declare test_id="e2e-gcloud-test-${RANDOM}"
declare api_token="e2e-API-token^^"
declare password="${RANDOM}${RANDOM}${RANDOM}"
declare wait_delay=2
declare wait_max_wait=1000

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT

  # Cleaning up everything
  gcloud_delete_managed_instance_group "${test_id}"
  gcloud_delete_instance_template "${test_id}"

  # TODO: delete hardhat instance

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# $1 = endpoint
function fund_node() {
  local endpoint=${1}

  local eth_address
  eth_address="$(curl --silent "${endpoint}/api/v1/address/hopr")"

  if [ -z "${eth_address}" ]; then
    log "Can't fund node - couldn't load ETH address"
    exit 1
  fi

  log "Funding 1 ETH and 1 HOPR to ${eth_address}"
  yarn hardhat faucet --config packages/ethereum/hardhat.config.ts \
    --address "${eth_address}" --network "${rpc_endpoint_ip}" --ishopraddress true
}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
DEVELOPMENT=true yarn hardhat node --config packages/ethereum/hardhat.config.ts \
  --network hardhat --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &
# TODO: start hardhat instance

rpc_endpoint="1.1.1.1:8545"

log "Hardhat node started (${rpc_endpoint})"
wait_for_http_port 8545 "" "${wait_delay}" "${wait_max_wait}"
# }}}

gcloud_create_or_update_instance_template "${test_id}" \
  "${docker_image}" \
  "${rpc_endpoint}}" \
  "${api_token}" \
  "${password}"

gcloud_create_or_update_managed_instance_group "${test_id}" \
  6 \
  "${test_id}"

# get IPs of newly started VMs which run hoprd
local node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${test_id}")

#  --- Fund nodes --- {{{
for ip in ${node_ips}; do
  fund_node "${ip}:3001"
done
# }}}

#  --- Wait for nodes to register funding and complete startup--- {{{
for ip in ${node_ips}; do
  wait_for_port "${ip}" "9091"
done
# }}}

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  "${node_ips%% *}" 3001 9091
#}}}

# --- Run test --- {{{
${mydir}/../test/integration-test.sh \
  ${node_ips// /:3001 }:3001
# }}}
