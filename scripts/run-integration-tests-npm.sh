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
  msg "Usage: $0 [-h|--help] [-v|--package-version <npm_package_version>] [-s|--skip-cleanup]"
  msg
  msg "If <npm_package_version> is not given, the local version of hoprd will be packaged and tested."
  msg "The cleanup process can be skipped by using '--skip-cleanup'."
  msg
}

# verify and set parameters
declare wait_delay=2
declare wait_max_wait=1000
declare cwd=`pwd`
declare npm_package_version=""
declare skip_cleanup="false"
declare api_token="e2e-API-token^^"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -v|--package-version)
      npm_package_version="$2"
      # remove prefix 'v' if it exists
      npm_package_version=${npm_package_version#v}
      shift 2
      ;;
    -s|--skip-cleanup)
      skip_cleanup="true"
      shift
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


if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  wait_delay=10
  wait_max_wait=10
fi

# find usable tmp dir
declare tmp="/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && tmp="/var/tmp"
[[ -d "${tmp}" && -h "${tmp}" ]] && { msg "Neither /tmp or /var/tmp can be used for writing logs"; exit 1; }

declare npm_install_dir="${tmp}/hopr-npm"

declare node_prefix="hopr-npm-node"

declare node1_dir="${tmp}/${node_prefix}-1"
declare node2_dir="${tmp}/${node_prefix}-2"
declare node3_dir="${tmp}/${node_prefix}-3"
declare node4_dir="${tmp}/${node_prefix}-4"
declare node5_dir="${tmp}/${node_prefix}-5"
declare node6_dir="${tmp}/${node_prefix}-6"
declare node7_dir="${tmp}/${node_prefix}-7"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"
declare node5_log="${node5_dir}.log"
declare node6_log="${node6_dir}.log"
declare node7_log="${node7_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"
declare node5_id="${node5_dir}.id"
declare node6_id="${node6_dir}.id"
declare node7_id="${node7_dir}.id"

declare password="e2e-test"

declare hardhat_rpc_log="${tmp}/hopr-npm-hardhat-rpc.log"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${node3_dir}" "${node4_dir}" "${node5_dir}" "${node6_dir}" "${node7_dir}" "${npm_install_dir}"

  log "Cleaning up processes"
  for port in 8545 13301 13302 13303 13304 13305 13306 13307 19091 19092 19093 19094 19095 19096 19097 20000; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  log "Remove default environment setting"
  rm -f "${mydir}/../packages/hoprd/default-environment.json"

  local log exit_code non_zero
  for node_log in "${node1_log}" "${node2_log}" "${node3_log}" "${node4_log}" "${node5_log}" "${node6_log}" "${node7_log}"; do
    log=$(wait_for_regex ${node_log} "Process exiting with signal [0-9]")

    if [ -z "${log}" ]; then
      log "${node_log}"
      log "Process did not exit properly"
      exit 1
    fi

    exit_code=$(echo ${log} | sed -E "s/.*signal[ ]([0-9]+).*/\1/")
    if [ ${exit_code} != "0" ]; then
      non_zero=true
      log "${node_log}"
      log "terminated with non-zero exit code ${exit_code}"
    fi
  done

  if [ ${non_zero} ]; then
    exit 1
  else
    exit $EXIT_CODE
  fi
}

if [ "${skip_cleanup}" != "1" ] && [ "${skip_cleanup}" != "true" ]; then
  trap cleanup SIGINT SIGTERM ERR EXIT
fi

# $1 = api port
# $2 = node port
# $3 = admin port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = OPTIONAL: additional args to hoprd
function setup_node() {
  local api_port=${1}
  local node_port=${2}
  local admin_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local additional_args=${7:-""}

  log "Run node ${id} on API port ${api_port} -> ${log}"

  if [ -n "${additional_args}" ]; then
    log "Additional args: \"${additional_args}\""
  fi

  install_npm_packages
  cd "${npm_install_dir}"

  DEBUG="hopr*" npx hoprd \
    --admin \
    --adminHost "127.0.0.1" \
    --adminPort ${admin_port} \
    --api-token "${api_token}" \
    --data="${dir}" \
    --host="127.0.0.1:${node_port}" \
    --identity="${id}" \
    --init \
    --password="${password}" \
    --api \
    --apiPort "${api_port}" \
    --testAnnounceLocalAddresses \
    --testPreferLocalAddresses \
    --testUseWeakCrypto \
    --testNoUPNP \
    --allowLocalNodeConnections \
    --allowPrivateNodeConnections \
    --heartbeatInterval 2500 \
    --heartbeatVariance 1000 \
    ${additional_args} \
    > "${log}" 2>&1 &

  # back to our original directory
  cd "${cwd}"
}

function create_npm_package() {
  local lib="${1}"
  yarn workspace @hoprnet/${lib} pack
  mv ${mydir}/../packages/${lib#hopr-}/package.tgz "${tmp}/${lib}-package.tgz"
}

function install_npm_packages() {
  if [ -d "${npm_install_dir}" ]; then
    # noop. already setup
    :
  else
    # move into work dir before we proceed
    mkdir -p "${npm_install_dir}"
    cd "${npm_install_dir}"

    if [ -n "${npm_package_version}" ]; then
      npm install @hoprnet/hoprd@${npm_package_version}
    else
      # Some Node installations seems to miss node-pre-gyp
      npm install node-pre-gyp
      # Install modules according to their dependencies
      # @dev only works when cleaning node_modules afterwards,
      #      otherwise NPM might use outdated packages
      npm install ${tmp}/hopr-utils-package.tgz
      npm install ${tmp}/hopr-connect-package.tgz
      npm install ${tmp}/hopr-ethereum-package.tgz
      npm install ${tmp}/hopr-core-ethereum-package.tgz
      npm install ${tmp}/hopr-core-package.tgz
      npm install ${tmp}/hoprd-package.tgz
    fi

    # Copies local deployment information to npm install directory
    # Fixme: copy also other environments
    # need to mirror contract data because of hardhat-deploy node only writing to localhost
    log "Copying deployment information to npm directory (${npm_install_dir})"
    cp -R \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost/localhost" \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost/hardhat"
    cp -R \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
    cp -R \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
      "${npm_install_dir}/node_modules/@hoprnet/hopr-ethereum/deployments/hardhat-localhost"
    cp -R \
      "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
      "${npm_install_dir}/node_modules/@hoprnet/hopr-ethereum/deployments/hardhat-localhost2"
  fi
}

# --- Log test info {{{
if [ -n "${npm_package_version}" ]; then
  log "Using NPM package version: ${npm_package_version}"
fi
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
log "\tnode7"
log "\t\tdata dir: ${node7_dir} (will be removed)"
log "\t\tlog: ${node7_log}"
log "\t\tid: ${node7_id}"
# }}}

# --- Check all resources we need are free {{{
ensure_port_is_free 8545
ensure_port_is_free 13301
ensure_port_is_free 13302
ensure_port_is_free 13303
ensure_port_is_free 13304
ensure_port_is_free 13305
ensure_port_is_free 13306
ensure_port_is_free 13307
ensure_port_is_free 19091
ensure_port_is_free 19092
ensure_port_is_free 19093
ensure_port_is_free 19094
ensure_port_is_free 19095
ensure_port_is_free 19096
ensure_port_is_free 19097
# }}}

# --- Cleanup old contract deployments {{{
log "Removing artifacts from old contract deployments"
rm -Rf \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2" \
  "${npm_install_dir}/node_modules/@hoprnet/hopr-ethereum/deployments/hardhat-localhost" \
  "${npm_install_dir}/node_modules/@hoprnet/hopr-ethereum/deployments/hardhat-localhost2"
# }}}

#  --- Create packages if needed --- {{{
if [ -z "${npm_package_version}" ]; then
  create_npm_package "hopr-connect"
  create_npm_package "hopr-core"
  create_npm_package "hopr-utils"
  create_npm_package "hopr-ethereum"
  create_npm_package "hopr-core-ethereum"
  # set default environment
  echo '{"id": "hardhat-localhost"}' > "${mydir}/../packages/hoprd/default-environment.json"
  create_npm_package "hoprd"
fi
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
HOPR_ENVIRONMENT_ID="hardhat-localhost" \
TS_NODE_PROJECT=${mydir}/../packages/ethereum/tsconfig.hardhat.json \
yarn workspace @hoprnet/hopr-ethereum hardhat node \
  --network hardhat \
  --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server"
log "Hardhat node started (127.0.0.1:8545)"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 19091 19501 "${node1_dir}" "${node1_log}" "${node1_id}" "--announce"
setup_node 13302 19092 19502 "${node2_dir}" "${node2_log}" "${node2_id}" "--announce --testNoAuthentication"
setup_node 13303 19093 19503 "${node3_dir}" "${node3_log}" "${node3_id}" "--announce"
setup_node 13304 19094 19504 "${node4_dir}" "${node4_log}" "${node4_id}" "--testNoDirectConnections"
setup_node 13305 19095 19505 "${node5_dir}" "${node5_log}" "${node5_id}" "--testNoDirectConnections"
setup_node 13306 19096 19506 "${node6_dir}" "${node6_log}" "${node6_id}" "--announce --run \"info;balance\""
# should not be able to talk to the rest
setup_node 13307 19097 19507 "${node7_dir}" "${node7_log}" "${node7_id}" "--announce --environment hardhat-localhost2"
# }}}

# DO NOT MOVE THIS STEP
#  --- Wait until private key has been created or recovered --- {{{
wait_for_regex ${node1_log} "please fund this node"
wait_for_regex ${node2_log} "please fund this node"
wait_for_regex ${node3_log} "please fund this node"
wait_for_regex ${node4_log} "please fund this node"
wait_for_regex ${node5_log} "please fund this node"
wait_for_regex ${node6_log} "please fund this node"
wait_for_regex ${node7_log} "please fund this node"
# }}}

#  --- Fund nodes --- {{{
HOPR_ENVIRONMENT_ID=hardhat-localhost \
TS_NODE_PROJECT=${mydir}/../packages/ethereum/tsconfig.hardhat.json \
yarn workspace @hoprnet/hopr-ethereum hardhat faucet \
  --identity-prefix "${node_prefix}" \
  --identity-directory "${tmp}" \
  --use-local-identities \
  --network hardhat \
  --password "${password}"
# }}}

#  --- Wait for ports to be bound --- {{{
wait_for_regex ${node1_log} "STARTED NODE"
wait_for_regex ${node2_log} "STARTED NODE"
wait_for_regex ${node3_log} "STARTED NODE"
wait_for_regex ${node4_log} "STARTED NODE"
wait_for_regex ${node5_log} "STARTED NODE"
# no need to wait for node 6 since that will stop right away
wait_for_port 19097 "127.0.0.1" "${node7_log}"
# }}}

#  --- Ensure data directories are used --- {{{
for node_dir in ${node1_dir} ${node2_dir} ${node3_dir} ${node4_dir} ${node5_dir}; do
  declare node_dir_db="${node_dir}/db/LOG"
  declare node_dir_peerstore="${node_dir}/peerstore/LOG"
  [ -f "${node_dir_db}" ] || { echo "Data file ${node_dir_db} missing"; exit 1; }
  [ -f "${node_dir_peerstore}" ] || { echo "Data file ${node_dir_peerstore} missing"; exit 1; }
done
# }}}

# --- Run security tests --- {{{
${mydir}/../test/security-test.sh \
  127.0.0.1 13301 13302 19501 19502 "${api_token}"
#}}}

# --- Run test --- {{{
HOPRD_API_TOKEN="${api_token}" ${mydir}/../test/integration-test.sh \
  "localhost:13301" "localhost:13302" "localhost:13303" "localhost:13304" "localhost:13305" "localhost:13306" "localhost:13307"
# }}}

# -- Verify node6 has executed the commands {{{
log "Verifying node6 log output"
grep -E "HOPR Balance: +20000 txHOPR" "${node6_log}"
grep -E "ETH Balance: +10 xDAI" "${node6_log}"
grep -E "Running on: hardhat" "${node6_log}"
log "Output of node6 correct"
# }}}
