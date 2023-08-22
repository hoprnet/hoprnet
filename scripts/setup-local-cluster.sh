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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

PATH="${mydir}/../.foundry/bin:${mydir}/../.cargo/bin:${PATH}"

# verify and set parameters
declare api_token="^^LOCAL-testing-123^^"
declare myne_chat_url="http://app.myne.chat"
declare init_script=""
declare hoprd_command="node --experimental-wasm-modules ${mydir}/../packages/hoprd/lib/main.cjs"
declare listen_host="127.0.0.1"
declare node_env="development"
# first anvil account
declare deployer_private_key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-t|--api-token <api_token>] [-m|--myne-chat-url <myne_chat_url>] [-i|--init-script <init_script>] [--hoprd-command <hoprd_command>] [--listen-host|-l <list_host>] [-p|--production]"
  msg
  msg "<api_token> is set to '${api_token}' by default"
  msg "<myne_chat_url> is set to '${myne_chat_url}' by default"
  msg "<init_script> is empty by default, expected to be path to a script which is called with all node API endpoints as parameters"
  msg "<hoprd_command> is used to start hoprd, default is '${hoprd_command}'"
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
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done

declare tmp_dir node_prefix password anvil_rpc_log env_file
declare node_api_base_port node_p2p_base_port node_healthcheck_base_port
declare api_endpoints cluster_size
declare -a id_files

# find usable tmp dir
tmp_dir="$(find_tmp_dir)"

node_prefix="local"
password="local"
anvil_rpc_log="${tmp_dir}/hopr-local-anvil-rpc.log"
env_file="${tmp_dir}/local-cluster.env"
node_api_base_port=13301
node_p2p_base_port=19091
node_healthcheck_base_port=18081
cluster_size=5

function cleanup {
  local EXIT_CODE=$?

  # at this point we don't want to fail hard anymore
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Cleaning up everything
  log "Wiping databases"
  find -L "${tmp_dir}" -type d -name "${node_prefix}_*" -exec rm -rf {} +

  log "Cleaning up processes"
  for port in 8545 13301 13302 13303 13304 13305 19091 19092 19093 19094 19095; do
    lsof -i ":${port}" -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
  done

  log "Removing cluster env file"
  rm -f "${env_file}"

  exit $EXIT_CODE
}

trap cleanup SIGINT SIGTERM ERR EXIT

# $1 = node id
# $2 = host to listen on
# $3 = OPTIONAL: additional args to hoprd
function setup_node() {
  local node_id=${1?"setup_node requires node id"}
  local host=${2?"setup_node required host"}
  local additional_args=${3:-""}

  local api_port p2p_port healthcheck_port dir log id_file safe_args

  dir="${tmp_dir}/${node_prefix}_${node_id}"
  log="${dir}.log"
  id_file="${dir}.id"
  api_port=$(( node_api_base_port + node_id ))
  p2p_port=$(( node_p2p_base_port + node_id ))
  healthcheck_port=$(( node_healthcheck_base_port + node_id ))
  safe_args="$(<${dir}.safe.args)"

  api_endpoints+="localhost:${api_port} "

  log "Run node ${node_id} on rest port ${api_port} -> ${log}"

  if [[ "${additional_args}" != *"--network "* ]]; then
    additional_args="--network anvil-localhost ${additional_args}"
  fi

  log "safe args ${safe_args}"
  # read safe args and append to additional_args TODO:
  additional_args="${additional_args} ${safe_args}"

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
      --host="${host}:${p2p_port}" \
      --identity="${id_file}" \
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

function generate_local_identities() {
  log "Generate local identities"

  # remove existing identity files, .safe.args
  find -L "${tmp_dir}" -type f -name "${node_prefix}_*.safe.args" -delete
  find -L "${tmp_dir}" -type f -name "${node_prefix}_*.id" -delete

  env ETHERSCAN_API_KEY="" IDENTITY_PASSWORD="${password}" \
    hopli identity \
    --action create \
    --identity-directory "${tmp_dir}" \
    --identity-prefix "${node_prefix}_" \
    --number "${cluster_size}"
}

# read the identity file is located at $id_path
# create safe and module for each identity
function create_local_safes() {
  log "Create safe"

  mapfile -t id_files <<< "$(find -L "${tmp_dir}" -type f -name "${node_prefix}_*.id" | sort)"

  # create a loop so safes are created for all the nodes TODO:
  for id_file in ${id_files[@]}; do
    # store the returned `--safeAddress <safe_address> --moduleAddress <module_address>` to `safe_i.log` for each id
    # `hopli create-safe-module` will also add nodes to network registry and approve token transfers for safe
    env \
      ETHERSCAN_API_KEY="" \
      IDENTITY_PASSWORD="${password}" \
      PRIVATE_KEY="${deployer_private_key}" \
      DEPLOYER_PRIVATE_KEY="${deployer_private_key}" \
      hopli create-safe-module \
        --network anvil-localhost \
        --identity-from-path "${id_file}" \
        --contracts-root "./packages/ethereum/contracts" > "${id_file%.id}.safe.log"

    # store safe arguments in separate file for later use
    grep -oE "\--safeAddress.*--moduleAddress.*" "${id_file%.id}.safe.log" > "${id_file%.id}.safe.args"
    rm "${id_file%.id}.safe.log"
  done
}

# --- Log setup info {{{
log "Node files and directories"
log "\tanvil"
log "\t\tlog: ${anvil_rpc_log}"

for node_id in $(seq 0 $(( cluster_size - 1 ))); do
  declare id_file
  id_file="${tmp_dir}/${node_prefix}_${node_id}.id"
  log "\tnode ${node_prefix}_${node_id}"
  log "\t\tdata dir: ${id_file%.id} (will be removed)"
  log "\t\tlog: ${id_file/%.id/.log}"
  log "\t\tid: ${id_file}"
done
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

declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"

# --- Running Mock Blockchain --- {{{
log "Running anvil local node"
make -C "${mydir}/../" run-anvil args="-l ${anvil_rpc_log}"

log "Wait for anvil local node to complete startup"
wait_for_regex "${anvil_rpc_log}" "Listening on 0.0.0.0:8545"
log "Anvil node started (0.0.0.0:8545)"

# need to mirror contract data because of anvil-deploy node only writing to localhost
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost2"
# }}}

# create identity files to node1_id, .... node5_id
generate_local_identities

# create safe and modules for all the ids, store them in args files
create_local_safes

#  --- Run nodes --- {{{
for node_id in ${!id_files[@]}; do
  setup_node "${node_id}" "${listen_host}"
done

log "Waiting for nodes bootstrap"

for node_id in ${!id_files[@]}; do
  wait_for_regex "${tmp_dir}/${node_prefix}_${node_id}.log" "${listen_host}" "unfunded"
done
# }}}

log "Funding nodes"

#  --- Fund nodes --- {{{
make -C "${mydir}/../" fund-local-all \
  id_dir="${tmp_dir}"
# }}}

log "Waiting for nodes startup"

#  --- Wait until started --- {{{
# Wait until node has recovered its private key
for node_id in ${!id_files[@]}; do
  wait_for_regex "${tmp_dir}/${node_prefix}_${node_id}.log" "${listen_host}" "using blockchain address"
done
# }}}

log "Waiting for port binding"

#  --- Wait for ports to be bound --- {{{
for node_id in ${!id_files[@]}; do
  wait_for_regex "${tmp_dir}/${node_prefix}_${node_id}.log" "${listen_host}" "STARTED NODE"
done
# }}}

log "All nodes came up online"

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
    HOPRD_API_TOKEN="${api_token}" "${full_init_script}" "${api_endpoints}"
  else
    log "Error: Could not determine executable path of init script ${init_script}"
  fi
else
  log "No init script provided, skipping"
fi
# }}}

# --- Get peer ids for reporting --- {{{
declare -a peers
for endpoint in ${api_endpoints}; do
  declare peer
  peer="$(get_hopr_address "${api_token}@${endpoint}")"
  peers+=(${peer})
done
# }}}

log "NOTE:"
log ""
log "\tThe following links expect HOPR Admin to run at http://localhost:3000"
log "\tYou may use \`make run-hopr-admin\`"
log ""
log "Node port info"

cat <<EOF > "${env_file}"
#!/usr/bin/env bash
export apiToken="${api_token}"
EOF

for node_id in ${!id_files[@]}; do
  declare api_port node_name

  node_name="${node_prefix}_${node_id}"
  api_port=$(( node_api_base_port + node_id ))

  log "\t${node_name}"
  log "\t\tPeer Id:\t${peers[$node_id]}"
  log "\t\tRest API:\thttp://localhost:${api_port}/api/v3/_swagger"
  log "\t\tAdmin UI:\thttp://localhost:3000/?apiEndpoint=http://localhost:${api_port}&apiToken=${api_token}"
  log "\t\tHealthcheck:\thttp://localhost:$(( node_healthcheck_base_port + node_id ))/"
  log "\t\tWebSocket:\tws://localhost:${api_port}/api/v3/messages/websocket?apiToken=${api_token}"
  log "\t\tMyne Chat:\t${myne_chat_url}/?apiEndpoint=http://localhost:${api_port}&apiToken=${api_token}"

  cat <<EOF >> "${env_file}"
export HOPR_NODE_${node_id}_ADDR=${peers[$node_id]} HOPR_NODE_${node_id}_HTTP_URL=http://127.0.0.1:${api_port} HOPR_NODE_${node_id}_WS_URL=ws://127.0.0.1:${api_port}/api/v3/messages/websocket"
echo "---"
echo "Node ${node_name} REST API URL:  \$HOPR_NODE_${node_id}_HTTP_URL"
echo "Node ${node_name} WebSocket URL: \$HOPR_NODE_${node_id}_WS_URL"
echo "Node ${node_name} HOPR Address:  \$HOPR_NODE_${node_id}_ADDR"
echo "---"
EOF

done

if command -v gp; then
  gp sync-done "local-cluster"
else
  log "Run: 'source ${env_file}' in your shell to setup environment variables for this cluster (HOPR_NODE_1_ADDR, HOPR_NODE_1_HTTP_URL,... etc.)"
fi

log "Terminating this script will clean up the running local cluster"
trap - SIGINT SIGTERM ERR
wait
