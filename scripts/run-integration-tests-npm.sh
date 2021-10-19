#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-npm-test"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [<npm_package_version>]"
  msg
  msg "\twhere <npm_package_version> uses the most recent Git tag as default"
  msg
  msg "\tThe cleanup process can be skipped by setting the environment variable HOPRD_SKIP_CLEANUP to 'true'."
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare npm_package_version
declare skip_cleanup="${HOPRD_SKIP_CLEANUP:-false}"

# we rely on Git tags so need to fetch the tags in case they are not present
git fetch --unshallow --tags || :
npm_package_version=${1:-$(git describe --tags --abbrev=0)}
# remove prefix 'v' if it exists
npm_package_version=${npm_package_version#v}

declare wait_delay=2
declare wait_max_wait=1000
declare cwd=`pwd`

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

# find usable tmp dir
declare tmp="/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && tmp="/var/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && { msg "Neither /tmp or /var/tmp can be used for writing logs"; exit 1; }

declare npm_install_dir="${tmp}/hopr-npm"

declare node1_dir="${tmp}/hopr-npm-node-1"
declare node2_dir="${tmp}/hopr-npm-node-2"
declare node3_dir="${tmp}/hopr-npm-node-3"
declare node4_dir="${tmp}/hopr-npm-node-4"
declare node5_dir="${tmp}/hopr-npm-node-5"
declare node6_dir="${tmp}/hopr-npm-node-6"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"
declare node5_log="${node5_dir}.log"
declare node6_log="${node6_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"
declare node5_id="${node5_dir}.id"
declare node6_id="${node6_dir}.id"

declare hardhat_rpc_log="${tmp}/hopr-npm-hardhat-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${npm_install_dir}"

  log "Cleaning up processes"
  for port in 8545 13301 13302 13303 13304 13305 13306 19091 19092 19093 19094 19095 19096; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  exit $EXIT_CODE
}

if [ "${skip_cleanup}" != "1" ] && [ "${skip_cleanup}" != "true" ]; then
  trap cleanup SIGINT SIGTERM ERR EXIT
fi

# $1 = rest port
# $2 = node port
# $3 = admin port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = npm package version
# $8 = OPTIONAL: additions args to hoprd
function setup_node() {
  local rest_port=${1}
  local node_port=${2}
  local admin_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local version=${7}
  local additional_args=${8:-""}

  log "Run node ${id} on rest port ${rest_port}"

  if [ -n "${additional_args}" ]; then
    log "Additional args: \"${additional_args}\""
  fi

  # move into work dir before we proceed to use yarn
  mkdir -p "${npm_install_dir}"
  cd "${npm_install_dir}"

  npm install @hoprnet/hoprd@${version}

  # Copies local deployment information to npm install directory
  # Fixme: copy also other environments
  log "Copying deployment information to npm directory (${npm_install_dir})"
  cp -R ${cwd}/packages/ethereum/deployments/default/localhost ${npm_install_dir}/node_modules/@hoprnet/hopr-ethereum/deployments/default
  
  DEBUG="hopr*" npx hoprd \
    --admin \
    --adminHost "127.0.0.1" \
    --adminPort ${admin_port} \
    --announce \
    --api-token "e2e-API-token^^" \
    --data="${dir}" \
    --host="127.0.0.1:${node_port}" \
    --identity="${id}" \
    --init \
    --password="e2e-test" \
    --provider=http://127.0.0.1:8545/ \
    --rest \
    --restPort "${rest_port}" \
    --testAnnounceLocalAddresses \
    --testPreferLocalAddresses \
    --testUseWeakCrypto \
    ${additional_args} \
    > "${log}" 2>&1 &

  # back to our original directory
  cd "${cwd}"

  # Wait until node has recovered its private key
  wait_for_regex ${log} "using blockchain address"
}

# $1 = port
# $2 = node log file
function fund_node() {
  local port=${1}
  local log=${2}
  local api="127.0.0.1:${port}"

  local eth_address
  eth_address="$(curl --silent "${api}/api/v1/address/hopr")"

  if [ -z "${eth_address}" ]; then
    log "Can't fund node - couldn't load ETH address"
    exit 1
  fi

  log "Funding 1 ETH and 10 HOPR to ${eth_address}"
  yarn workspace @hoprnet/hopr-ethereum hardhat faucet \
    --address "${eth_address}" --network localhost --ishopraddress true --amount 10
}

# --- Log test info {{{
log "Using NPM package version: ${npm_package_version}"
log "Test files and directories"
log "\thardhat"
log "\t\tlog: ${hardhat_rpc_log}"
log "\tnpm package"
log "\t\tdir: ${npm_install_dir} (will be removed)"
log "\tnode1"
log "\t\tdata dir: ${node1_dir} (will be removed)"
log "\t\tlog: ${node1_log}"
log "\t\tid: ${node1_id}"
log "\tnode2"
log "\t\tdata dir: ${node2_dir} (will be removed)"
log "\t\tlog: ${node2_log}"
log "\t\tid: ${node2_id}"
log "\tnode3"
log "\t\tdata dir: ${node3_dir} (will be removed)"
log "\t\tlog: ${node3_log}"
log "\t\tid: ${node3_id}"
log "\tnode4"
log "\t\tdata dir: ${node4_dir} (will be removed)"
log "\t\tlog: ${node4_log}"
log "\t\tid: ${node4_id}"
log "\tnode5"
log "\t\tdata dir: ${node5_dir} (will be removed)"
log "\t\tlog: ${node5_log}"
log "\t\tid: ${node5_id}"
log "\tnode6"
log "\t\tdata dir: ${node6_dir} (will be removed)"
log "\t\tlog: ${node6_log}"
log "\t\tid: ${node6_id}"
# }}}

# --- Check all resources we need are free {{{
ensure_port_is_free 8545
ensure_port_is_free 13301
ensure_port_is_free 13302
ensure_port_is_free 13303
ensure_port_is_free 13304
ensure_port_is_free 13305
ensure_port_is_free 13306
ensure_port_is_free 19091
ensure_port_is_free 19092
ensure_port_is_free 19093
ensure_port_is_free 19094
ensure_port_is_free 19095
ensure_port_is_free 19096
# }}}

# --- Cleanup old deployments to localhost {{{
log "Removing artifacts from old deployments to localhost"
rm -Rfv packages/ethereum/deployments/*/localhost
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
DEVELOPMENT=true yarn workspace @hoprnet/hopr-ethereum hardhat node \
  --network hardhat --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server" 
log "Hardhat node started (127.0.0.1:8545)"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 19091 19501 "${node1_dir}" "${node1_log}" "${node1_id}" "${npm_package_version}"
setup_node 13302 19092 19502 "${node2_dir}" "${node2_log}" "${node2_id}" "${npm_package_version}" "--testNoAuthentication"
setup_node 13303 19093 19503 "${node3_dir}" "${node3_log}" "${node3_id}" "${npm_package_version}"
setup_node 13304 19094 19504 "${node4_dir}" "${node4_log}" "${node4_id}" "${npm_package_version}"
setup_node 13305 19095 19505 "${node5_dir}" "${node5_log}" "${node5_id}" "${npm_package_version}"
setup_node 13306 19096 19506 "${node6_dir}" "${node6_log}" "${node6_id}" "${npm_package_version}" "--run \"info;balance\""
# }}}

#  --- Fund nodes --- {{{
fund_node 13301 "${node1_log}"
fund_node 13302 "${node2_log}"
fund_node 13303 "${node3_log}"
fund_node 13304 "${node4_log}"
fund_node 13305 "${node5_log}"
fund_node 13306 "${node6_log}"
# }}}

#  --- Wait for ports to be bound --- {{{
wait_for_regex ${node1_log} "STARTED NODE"
wait_for_regex ${node2_log} "STARTED NODE"
wait_for_regex ${node3_log} "STARTED NODE"
wait_for_regex ${node4_log} "STARTED NODE"
wait_for_regex ${node5_log} "STARTED NODE"
# no need to wait for node 6 since that will stop right away
# }}}

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 13301 19501 19502
#}}}

# --- Run test --- {{{
${mydir}/../test/integration-test.sh \
  "localhost:13301" "localhost:13302" "localhost:13303" "localhost:13304" "localhost:13305"
# }}}

# -- Verify node6 has executed the commands {{{
log "Verifying node6 log output"
grep -E "^HOPR Balance: +10 HOPR$" "${node6_log}"
grep -E "^ETH Balance: +1 xDAI$" "${node6_log}"
grep -E "^Running on: localhost$" "${node6_log}"
# }}}
