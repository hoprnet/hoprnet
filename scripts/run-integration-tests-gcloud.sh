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
declare -x HOPR_LOG_ID="e2e-gcloud-test"
source "${mydir}/utils.sh"
source "${mydir}/gcloud.sh"
source "${mydir}/testnet.sh"

usage() {
  msg
  msg "Usage: $0 [<test_id> [<docker_image>]]"
  msg
  msg "where <test_id>:\t\tuses a random value as default"
  msg "      <docker_image>:\t\tuses 'gcr.io/hoprassociation/hoprd:latest' as default"
  msg
  msg "Required environment variables"
  msg "------------------------------"
  msg
  msg "FUNDING_PRIV_KEY\t\tsets the account which is used to fund nodes"
  msg
  msg "Optional environment variables"
  msg "------------------------------"
  msg
  msg "HOPRD_API_TOKEN\t\t\tused as api token for all nodes, defaults to a random value"
  msg "HOPRD_PASSWORD\t\t\tused as password for all nodes, defaults to a random value"
  msg "HOPRD_PROVIDER\t\t\tused as provider for all nodes, defaults to infura/goerli"
  msg "              \t\t\twhich requires the additional env var HOPRD_INFURA_KEY to be set"
  msg "HOPRD_RUN_CLEANUP_ONLY\t\tset to 'true' to execute the cleanup process only"
  msg "HOPRD_SHOW_PRESTART_INFO\tset to 'true' to print used parameter values before starting"
  msg "HOPRD_SKIP_CLEANUP\t\tset to 'true' to skip the cleanup process and keep resources running"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
declare test_id="e2e-gcloud-test-${1:-$RANDOM-$RANDOM}"
declare docker_image=${2:-gcr.io/hoprassociation/hoprd:latest}

declare api_token="${HOPRD_API_TOKEN:-Token${RANDOM}%${RANDOM}%${RANDOM}Token}"
declare password="${HOPRD_PASSWORD:-pw${RANDOM}${RANDOM}${RANDOM}pw}"
declare provider="${HOPRD_PROVIDER:-https://goerli.infura.io/v3/${HOPRD_INFURA_KEY}}"
declare skip_cleanup="${HOPRD_SKIP_CLEANUP:-false}"
declare show_prestartinfo="${HOPRD_SHOW_PRESTART_INFO:-false}"
declare run_cleanup_only="${HOPRD_RUN_CLEANUP_ONLY:-false}"

test -z "${FUNDING_PRIV_KEY:-}" && { msg "Missing FUNDING_PRIV_KEY"; usage; exit 1; }

function cleanup {
  local EXIT_CODE=$?

  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  gcloud_delete_managed_instance_group "${test_id}"
  gcloud_delete_instance_template "${test_id}"

  exit $EXIT_CODE
}

function fund_ip() {
  local ip="${1}"
  local eth_address

  wait_until_node_is_ready "${ip}"
  eth_address=$(get_native_address "${ip}:3001")
  fund_if_empty "${eth_address}" "${provider}"
  wait_for_port "9091" "${ip}"
}

if [ "${run_cleanup_only}" = "1" ] || [ "${run_cleanup_only}" = "true" ]; then
  cleanup

  # exit right away
  exit
fi

if [ "${skip_cleanup}" != "1" ] && [ "${skip_cleanup}" != "true" ]; then
  trap cleanup SIGINT SIGTERM ERR EXIT
fi

# --- Log test info {{{
if [ "${show_prestartinfo}" = "1" ] || [ "${show_prestartinfo}" = "true" ]; then
  log "Pre-Start Info"
  log "\tdocker_image: ${docker_image}"
  log "\ttest_id: ${test_id}"
  log "\tapi_token: ${api_token}"
  log "\tpassword: ${password}"
  log "\tprovider: ${provider}"
  log "\tskip_cleanup: ${skip_cleanup}"
  log "\tshow_prestartinfo: ${show_prestartinfo}"
  log "\trun_cleanup_only: ${run_cleanup_only}"
fi
# }}}

# create test specific instance template
gcloud_create_or_update_instance_template "${test_id}" \
  "${docker_image}" \
  "${provider}" \
  "${api_token}" \
  "${password}"
#
# start nodes
gcloud_create_or_update_managed_instance_group "${test_id}" \
  6 \
  "${test_id}"

# get IPs of newly started VMs which run hoprd
declare node_ips
node_ips=$(gcloud_get_managed_instance_group_instances_ips "${test_id}")
declare node_ips_arr=( ${node_ips} )

#  --- Fund nodes --- {{{
declare eth_address
for ip in ${node_ips}; do
  wait_until_node_is_ready "${ip}"
  eth_address=$(get_native_address "${ip}:3001")
  fund_if_empty "${eth_address}" "${provider}"
done

for ip in ${node_ips}; do
  wait_for_port "9091" "${ip}"
done
# }}}

# --- Run security tests --- {{{
"${mydir}/../test/security-test.sh" \
  "${node_ips_arr[0]}" 3001 3000
#}}}

# --- Run test --- {{{
"${mydir}/../test/integration-test.sh" \
  ${node_ips_arr[@]/%/:3001}
# }}}
