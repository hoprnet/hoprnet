#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="setup-local-cluster"
source "${mydir}/utils.sh"

# verify and set parameters
declare api_token="^^LOCAL-testing-123^^"
declare myne_chat_url="http://app.myne.chat"
declare init_script=""

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-t|--api-token <api_token>] [-m|--myne-chat-url <myne_chat_url>] [-i|--init-script <init_script>]"
  msg
  msg "<api_token> is set to '${api_token}' by default"
  msg "<myne_chat_url> is set to '${myne_chat_url}' by default"
  msg "<init_script> is empty by default, expected to be path to a script which is called with all node API endpoints as parameters"
}

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -t|--api-token)
      api_token="${2}"
      shift
      shift
      ;;
    -m|--myne-chat-url)
      myne_chat_url="${2}"
      shift
      shift
      ;;
    -i|--init-script)
      init_script="${2}"
      shift
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

# find usable tmp dir
declare tmp="$(find_tmp_dir)"

declare node_prefix="local"

declare node1_dir="${tmp}/${node_prefix}-1"
declare node2_dir="${tmp}/${node_prefix}-2"
declare node3_dir="${tmp}/${node_prefix}-3"
declare node4_dir="${tmp}/${node_prefix}-4"
declare node5_dir="${tmp}/${node_prefix}-5"

declare node1_log="${node1_dir}.log"
declare node2_log="${node2_dir}.log"
declare node3_log="${node3_dir}.log"
declare node4_log="${node4_dir}.log"
declare node5_log="${node5_dir}.log"

declare node1_id="${node1_dir}.id"
declare node2_id="${node2_dir}.id"
declare node3_id="${node3_dir}.id"
declare node4_id="${node4_dir}.id"
declare node5_id="${node5_dir}.id"

declare password="local"

declare hardhat_rpc_log="${tmp}/hopr-local-hardhat-rpc.log"
declare env_file="${tmp}/local-cluster.env"

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Wiping databases"
  rm -rf "${node1_dir}" "${node2_dir}" "${node3_dir}" "${node4_dir}" "${node5_dir}" "${node7_dir}"

  log "Cleaning up processes"
  for port in 8545 13301 13302 13303 13304 13305 19091 19092 19093 19094 19095; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  rm ${env_file}

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# $1 = rest port
# $2 = node port
# $3 = admin port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = OPTIONAL: additional args to hoprd
function setup_node() {
  local rest_port=${1}
  local node_port=${2}
  local admin_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local additional_args=${7:-""}

  log "Run node ${id} on rest port ${rest_port} -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment hardhat-localhost ${additional_args}"
  fi

  log "Additional args: \"${additional_args}\""

  # Set NODE_ENV=development to rebuild hopr-admin next files
  # at runtime. Necessary to start multiple instances of hoprd
  # in parallel
  DEBUG="hopr*" NODE_ENV=development node packages/hoprd/lib/index.js \
    --admin \
    --adminHost "127.0.0.1" \
    --adminPort ${admin_port} \
    --announce \
    --api-token "${api_token}" \
    --data="${dir}" \
    --host="127.0.0.1:${node_port}" \
    --identity="${id}" \
    --init \
    --password="${password}" \
    --rest \
    --restPort "${rest_port}" \
    --testAnnounceLocalAddresses \
    --testPreferLocalAddresses \
    --testUseWeakCrypto \
    --allowLocalNodeConnections \
    --allowPrivateNodeConnections \
    ${additional_args} \
    > "${log}" 2>&1 &
}

# --- Log setup info {{{
log "Node files and directories"
log "\thardhat"
log "\t\tlog: ${hardhat_rpc_log}"
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
# }}}

# --- Check all resources we need are free {{{
ensure_port_is_free 8545
ensure_port_is_free 13301
ensure_port_is_free 13302
ensure_port_is_free 13303
ensure_port_is_free 13304
ensure_port_is_free 13305
ensure_port_is_free 19091
ensure_port_is_free 19092
ensure_port_is_free 19093
ensure_port_is_free 19094
ensure_port_is_free 19095
# }}}

# --- Cleanup old contract deployments {{{
log "Removing artifacts from old contract deployments"
rm -Rfv \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
HOPR_ENVIRONMENT_ID="hardhat-localhost" yarn workspace @hoprnet/hopr-ethereum hardhat node \
  --network hardhat --show-stack-traces > \
  "${hardhat_rpc_log}" 2>&1 &

wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server"
log "Hardhat node started (127.0.0.1:8545)"

# need to mirror contract data because of hardhat-deploy node only writing to localhost
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost/hardhat"
cp -R \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost" \
  "${mydir}/../packages/ethereum/deployments/hardhat-localhost2"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 19091 19501 "${node1_dir}" "${node1_log}" "${node1_id}"
setup_node 13302 19092 19502 "${node2_dir}" "${node2_log}" "${node2_id}"
setup_node 13303 19093 19503 "${node3_dir}" "${node3_log}" "${node3_id}"
setup_node 13304 19094 19504 "${node4_dir}" "${node4_log}" "${node4_id}"
setup_node 13305 19095 19505 "${node5_dir}" "${node5_log}" "${node5_id}"
# }}}

log "Waiting for nodes bootstrap"

wait_for_regex ${node1_log} "unfunded"
wait_for_regex ${node2_log} "unfunded"
wait_for_regex ${node3_log} "unfunded"
wait_for_regex ${node4_log} "unfunded"
wait_for_regex ${node5_log} "unfunded"

log "Funding nodes"

#  --- Fund nodes --- {{{
HOPR_ENVIRONMENT_ID=hardhat-localhost yarn workspace @hoprnet/hopr-ethereum hardhat faucet \
  --identity-prefix "${node_prefix}" \
  --identity-directory "${tmp}" \
  --use-local-identities \
  --network hardhat \
  --password "${password}"
# }}}

log "Waiting for nodes startup"

#  --- Wait until started --- {{{
# Wait until node has recovered its private key
wait_for_regex ${node1_log} "using blockchain address"
wait_for_regex ${node2_log} "using blockchain address"
wait_for_regex ${node3_log} "using blockchain address"
wait_for_regex ${node4_log} "using blockchain address"
wait_for_regex ${node5_log} "using blockchain address"
# }}}

log "Waiting for port binding"

#  --- Wait for ports to be bound --- {{{
wait_for_regex ${node1_log} "STARTED NODE"
wait_for_regex ${node2_log} "STARTED NODE"
wait_for_regex ${node3_log} "STARTED NODE"
wait_for_regex ${node4_log} "STARTED NODE"
wait_for_regex ${node5_log} "STARTED NODE"
# }}}

log "All nodes came up online"

declare endpoints="localhost:13301 localhost:13302 localhost:13303 localhost:13304 localhost:13305"

# --- Call init script--- {{{
if [ -n "${init_script}" ] && [ -x "${init_script}" ]; then
  log "Calling init script ${init_script}"
  HOPRD_API_TOKEN="${api_token}" \
    "${init_script}" ${endpoints}
fi
# }}}

# --- Get peer ids for reporting --- {{{
declare -a peers
for endpoint in ${endpoints}; do
  declare peer="$(get_hopr_address "${api_token}@${endpoint}")"
  peers+=(${peer})
done
# }}}

log "Node port info"
log "\tnode1"
log "\t\tPeer Id:\t${peers[0]}"
log "\t\tRest API:\thttp://localhost:13301/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19501/"
log "\t\tMyne Chat:\t${myne_chat_url}/?httpEndpoint=http://localhost:13301&wsEndpoint=ws://localhost:19501&securityToken=${api_token}"
log "\tnode2"
log "\t\tPeer Id:\t${peers[1]}"
log "\t\tRest API:\thttp://localhost:13302/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19502/"
log "\t\tMyne Chat:\t${myne_chat_url}/?httpEndpoint=http://localhost:13302&wsEndpoint=ws://localhost:19502&securityToken=${api_token}"
log "\tnode3"
log "\t\tPeer Id:\t${peers[2]}"
log "\t\tRest API:\thttp://localhost:13303/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19504/"
log "\t\tMyne Chat:\t${myne_chat_url}/?httpEndpoint=http://localhost:13303&wsEndpoint=ws://localhost:19503&securityToken=${api_token}"
log "\tnode4"
log "\t\tPeer Id:\t${peers[3]}"
log "\t\tRest API:\thttp://localhost:13304/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19504/"
log "\t\tMyne Chat:\t${myne_chat_url}/?httpEndpoint=http://localhost:13304&wsEndpoint=ws://localhost:19504&securityToken=${api_token}"
log "\tnode5"
log "\t\tPeer Id:\t${peers[4]}"
log "\t\tRest API:\thttp://localhost:13305/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19505/"
log "\t\tMyne Chat:\t${myne_chat_url}/?httpEndpoint=http://localhost:13305&wsEndpoint=ws://localhost:19505&securityToken=${api_token}"

cat <<EOF > ${env_file}
#!/usr/bin/env bash
export apiToken="${api_token}"
export HOPR_NODE_1_ADDR=${peers[0]} HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:19501
export HOPR_NODE_2_ADDR=${peers[1]} HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=ws://127.0.0.1:19502
export HOPR_NODE_3_ADDR=${peers[2]} HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=ws://127.0.0.1:19503
export HOPR_NODE_4_ADDR=${peers[3]} HOPR_NODE_4_HTTP_URL=http://127.0.0.1:13304 HOPR_NODE_4_WS_URL=ws://127.0.0.1:19504
export HOPR_NODE_5_ADDR=${peers[4]} HOPR_NODE_5_HTTP_URL=http://127.0.0.1:13305 HOPR_NODE_5_WS_URL=ws://127.0.0.1:19505
echo -e "\n"
echo "🌐 Node 1 REST API URL:  \$HOPR_NODE_1_HTTP_URL"
echo "🔌 Node 1 WebSocket URL: \$HOPR_NODE_1_WS_URL"
echo "💻 Node 1 HOPR Address:  \$HOPR_NODE_1_ADDR"
echo "---" 
echo "🌐 Node 2 REST API URL:  \$HOPR_NODE_2_HTTP_URL"
echo "🔌 Node 2 WebSocket URL: \$HOPR_NODE_2_WS_URL"
echo "💻 Node 2 HOPR Address:  \$HOPR_NODE_2_ADDR"
echo "---" 
echo "🌐 Node 3 REST API URL:  \$HOPR_NODE_3_HTTP_URL"
echo "🔌 Node 3 WebSocket URL: \$HOPR_NODE_3_WS_URL"
echo "💻 Node 3 HOPR Address:  \$HOPR_NODE_3_ADDR"
echo "---"
echo "🌐 Node 4 REST API URL:  \$HOPR_NODE_4_HTTP_URL"
echo "🔌 Node 4 WebSocket URL: \$HOPR_NODE_4_WS_URL"
echo "💻 Node 4 HOPR Address:  \$HOPR_NODE_4_ADDR"
echo "---" ;
echo "🌐 Node 5 REST API URL:  \$HOPR_NODE_5_HTTP_URL"
echo "🔌 Node 5 WebSocket URL: \$HOPR_NODE_5_WS_URL"
echo "💻 Node 5 HOPR Address:  \$HOPR_NODE_5_ADDR"
echo -e "\n"
EOF

# GitPod related barrier
if command -v gp; then
  gp sync-done "local-cluster"
else
  log "Run: 'source ${env_file}' in your shell to setup environment variables for this cluster (HOPR_NODE_1_ADDR, HOPR_NODE_1_HTTP_URL,... etc.)"
fi

log "Terminating this script will clean up the running local cluster"
wait
