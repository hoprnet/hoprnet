#!/usr/bin/env bash

# TODO: this script currently uses goerli as the RPC provider. However, it
# should be extended to use its own instance of hardhat too.

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
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "Usage: $0 [<docker_image>] [<test_id>]"
  msg
  msg "\twhere <docker_image>: uses 'gcr.io/hoprassociation/hoprd:latest' as default"
  msg "\t      <test_id>: uses a random value as default"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare docker_image test_id
docker_image=${1:-gcr.io/hoprassociation/hoprd:latest}
test_id="e2e-gcloud-test-${2:-$RANDOM-$RANDOM}"

declare api_token="e2e-API-token^^"
declare password="pw${RANDOM}${RANDOM}${RANDOM}pw"
declare wait_delay=2
declare wait_max_wait=1000
declare rpc_endpoint="https://goerli.infura.io/v3/${HOPRD_INFURA_KEY}"
declare hopr_token_contract="0x566a5c774bb8ABE1A88B4f187e24d4cD55C207A5"

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  gcloud_delete_managed_instance_group "${test_id}"
  gcloud_delete_instance_template "${test_id}"

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# create test specific instance template
gcloud_create_or_update_instance_template "${test_id}" \
  "${docker_image}" \
  "${rpc_endpoint}" \
  "${api_token}" \
  "${password}"
#
# start nodes
gcloud_create_or_update_managed_instance_group "${test_id}" \
  6 \
  "${test_id}"

# get IPs of newly started VMs which run hoprd
declare node_ips node_ips_arr
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${test_id}")
node_ips_arr=( ${node_ips} )

#  --- Fund nodes --- {{{
for ip in ${node_ips}; do
  wait_until_node_is_ready "${ip}"
  declare eth_address=$(get_eth_address "${ip}")
  fund_if_empty "${eth_address}" "${rpc_endpoint}" "${hopr_token_contract}"
done
# }}}

#  --- Wait for nodes to register funding and complete startup--- {{{
for ip in ${node_ips}; do
  wait_for_port "9091" "${ip}"
done
# }}}

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  "${node_ips_arr[0]}" 3001 3000
#}}}

# --- Run test --- {{{
${mydir}/../test/integration-test.sh \
  ${node_ips_arr[@]/%/:3001}
# }}}
