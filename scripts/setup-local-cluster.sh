#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
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
declare hoprd_command="node --experimental-wasm-modules packages/hoprd/lib/main.cjs"
declare hardhat_basedir="packages/ethereum"
declare listen_host="127.0.0.1"
declare node_env="development"

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-t|--api-token <api_token>] [-m|--myne-chat-url <myne_chat_url>] [-i|--init-script <init_script>] [--hoprd-command <hoprd_command>] [--hardhat-basedir <hardhat_basedir>] [--listen-host|-l <list_host>] [-p|--production]"
  msg
  msg "<api_token> is set to '${api_token}' by default"
  msg "<myne_chat_url> is set to '${myne_chat_url}' by default"
  msg "<init_script> is empty by default, expected to be path to a script which is called with all node API endpoints as parameters"
  msg "<hoprd_command> is used to start hoprd, default is '${hoprd_command}'"
  msg "<hardhat_basedir> is entered before hardhat is started, default is '${hardhat_basedir}'"
  msg "<listen_host> is listened on by all hoprd instances, default is '${listen_host}'"
  msg "-p sets NODE_ENV to 'production'"
}

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -p|--production)
      node_env="production"
      shift
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
    -l|--listen-host)
      listen_host="${2}"
      shift
      shift
      ;;
    --hoprd-command)
      hoprd_command="${2}"
      shift
      shift
      ;;
    --hardhat-basedir)
      hardhat_basedir="${2}"
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
declare tmp_dir="$(find_tmp_dir)"

declare node_prefix="local"

declare node1_dir="${tmp_dir}/${node_prefix}-1"
declare node2_dir="${tmp_dir}/${node_prefix}-2"
declare node3_dir="${tmp_dir}/${node_prefix}-3"
declare node4_dir="${tmp_dir}/${node_prefix}-4"
declare node5_dir="${tmp_dir}/${node_prefix}-5"

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

declare hardhat_rpc_log="${tmp_dir}/hopr-local-hardhat-rpc.log"
declare env_file="${tmp_dir}/local-cluster.env"

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

  log "Removing cluster env file"
  rm -f ${env_file}

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# $1 = rest port
# $2 = node port
# $3 = healthcheck port
# $4 = node data directory
# $5 = node log file
# $6 = node id file
# $7 = host to listen on
# $8 = OPTIONAL: additional args to hoprd
function setup_node() {
  local api_port=${1}
  local node_port=${2}
  local healthcheck_port=${3}
  local dir=${4}
  local log=${5}
  local id=${6}
  local host=${7}
  local additional_args=${8:-""}

  log "Run node ${id} on rest port ${api_port} -> ${log}"

  if [[ "${additional_args}" != *"--environment "* ]]; then
    additional_args="--environment hardhat-localhost ${additional_args}"
  fi

  log "Additional args: \"${additional_args}\""

  env \
    DEBUG="hopr*" \
    NODE_ENV="${node_env}" \
    HOPRD_HEARTBEAT_INTERVAL=2500 \
    HOPRD_HEARTBEAT_THRESHOLD=2500 \
    HOPRD_HEARTBEAT_VARIANCE=1000 \
    HOPRD_NETWORK_QUALITY_THRESHOLD="0.3" \
    HOPRD_ON_CHAIN_CONFIRMATIONS=2 \
    ${hoprd_command} \
      --announce \
      --api-token "${api_token}" \
      --data="${dir}" \
      --host="${host}:${node_port}" \
      --identity="${id}" \
      --init \
      --password="${password}" \
      --api \
      --apiHost "${host}" \
      --apiPort "${api_port}" \
      --testAnnounceLocalAddresses \
      --testPreferLocalAddresses \
      --testUseWeakCrypto \
      --allowLocalNodeConnections \
      --allowPrivateNodeConnections \
      --healthCheck \
      --healthCheckHost "${host}" \
      --healthCheckPort "${healthcheck_port}" \
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
  "${hardhat_basedir}/deployments/hardhat-localhost/*" \
  "${hardhat_basedir}/deployments/hardhat-localhost2"
# }}}

# --- Running Mock Blockchain --- {{{
log "Running hardhat local node"
cd "${hardhat_basedir}" && \
  NODE_OPTIONS=--experimental-wasm-modules yarn run:network \
    --network hardhat --show-stack-traces > \
    "${hardhat_rpc_log}" 2>&1 &

wait_for_regex ${hardhat_rpc_log} "Started HTTP and WebSocket JSON-RPC server"
log "Hardhat node started (127.0.0.1:8545)"

# need to mirror contract data because of hardhat-deploy node only writing to localhost
cp -R \
  "${hardhat_basedir}/deployments/hardhat-localhost/localhost" \
  "${hardhat_basedir}/deployments/hardhat-localhost/hardhat"
cp -R \
  "${hardhat_basedir}/deployments/hardhat-localhost" \
  "${hardhat_basedir}/deployments/hardhat-localhost2"
# }}}

#  --- Run nodes --- {{{
setup_node 13301 19091 18081 "${node1_dir}" "${node1_log}" "${node1_id}" "${listen_host}"
setup_node 13302 19092 18082 "${node2_dir}" "${node2_log}" "${node2_id}" "${listen_host}"
setup_node 13303 19093 18083 "${node3_dir}" "${node3_log}" "${node3_id}" "${listen_host}"
setup_node 13304 19094 18084 "${node4_dir}" "${node4_log}" "${node4_id}" "${listen_host}"
setup_node 13305 19095 18085 "${node5_dir}" "${node5_log}" "${node5_id}" "${listen_host}"
# }}}

log "Waiting for nodes bootstrap"

wait_for_regex ${node1_log} "unfunded"
wait_for_regex ${node2_log} "unfunded"
wait_for_regex ${node3_log} "unfunded"
wait_for_regex ${node4_log} "unfunded"
wait_for_regex ${node5_log} "unfunded"

log "Funding nodes"

#  --- Fund nodes --- {{{
cd "${hardhat_basedir}" && \
  NODE_OPTIONS=--experimental-wasm-modules yarn faucet --identity-directory "${tmp_dir}"
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
if [ -n "${init_script}" ]; then
  declare full_init_script=""

  # find full executable path of the script
  if [ -x "${init_script}" ]; then
    full_init_script="${init_script}"
  elif [ -x "${mydir}/${init_script}" ]; then
    full_init_script="${mydir}/${init_script}"
  fi

  # execute script if a path was found
  if [ -n "${full_init_script}" ]; then
    log "Calling init script ${full_init_script}"
    HOPRD_API_TOKEN="${api_token}" "${full_init_script}" ${endpoints}
  else
    log "Error: Could not determine executable path of init script ${init_script}"
  fi
else
  log "No init script provided, skipping"
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
log "\t\tAdmin UI:\thttp://localhost:19501/?apiEndpoint=http://localhost:13301&apiToken=${api_token}"
log "\t\tHealthcheck:\thttp://localhost:18081/"
log "\t\tWebSocket:\tws://localhost:13301/api/v2/messages/websocket?apiToken=${api_token}"
log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:13301&apiToken=${api_token}"
log "\tnode2"
log "\t\tPeer Id:\t${peers[1]}"
log "\t\tRest API:\thttp://localhost:13302/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19502/?apiEndpoint=http://localhost:13302&apiToken=${api_token}"
log "\t\tHealthcheck:\thttp://localhost:18082/"
log "\t\tWebSocket:\tws://localhost:13302/api/v2/messages/websocket?apiToken=${api_token}"
log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:13302&apiToken=${api_token}"
log "\tnode3"
log "\t\tPeer Id:\t${peers[2]}"
log "\t\tRest API:\thttp://localhost:13303/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19503/?apiEndpoint=http://localhost:13303&apiToken=${api_token}"
log "\t\tHealthcheck:\thttp://localhost:18083/"
log "\t\tWebSocket:\tws://localhost:13303/api/v2/messages/websocket?apiToken=${api_token}"
log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:13303&apiToken=${api_token}"
log "\tnode4"
log "\t\tPeer Id:\t${peers[3]}"
log "\t\tRest API:\thttp://localhost:13304/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19504/?apiEndpoint=http://localhost:13304&apiToken=${api_token}"
log "\t\tHealthcheck:\thttp://localhost:18084/"
log "\t\tWebSocket:\tws://localhost:13304/api/v2/messages/websocket?apiToken=${api_token}"
log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:13304&apiToken=${api_token}"
log "\tnode5"
log "\t\tPeer Id:\t${peers[4]}"
log "\t\tRest API:\thttp://localhost:13305/api/v2/_swagger"
log "\t\tAdmin UI:\thttp://localhost:19505/?apiEndpoint=http://localhost:13305&apiToken=${api_token}"
log "\t\tHealthcheck:\thttp://localhost:18085/"
log "\t\tWebSocket:\tws://localhost:13305/api/v2/messages/websocket?apiToken=${api_token}"
log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:13305&apiToken=${api_token}"

cat <<EOF > ${env_file}
#!/usr/bin/env bash
export apiToken="${api_token}"
export HOPR_NODE_1_ADDR=${peers[0]} HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:13301/api/v2/messages/websocket
export HOPR_NODE_2_ADDR=${peers[1]} HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=ws://127.0.0.1:13302/api/v2/messages/websocket
export HOPR_NODE_3_ADDR=${peers[2]} HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=ws://127.0.0.1:13303/api/v2/messages/websocket
export HOPR_NODE_4_ADDR=${peers[3]} HOPR_NODE_4_HTTP_URL=http://127.0.0.1:13304 HOPR_NODE_4_WS_URL=ws://127.0.0.1:13304/api/v2/messages/websocket
export HOPR_NODE_5_ADDR=${peers[4]} HOPR_NODE_5_HTTP_URL=http://127.0.0.1:13305 HOPR_NODE_5_WS_URL=ws://127.0.0.1:13305/api/v2/messages/websocket
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
trap - SIGINT SIGTERM ERR
wait
