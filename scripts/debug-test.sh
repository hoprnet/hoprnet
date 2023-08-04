#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

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

  if [[ "${additional_args}" != *"--network "* ]]; then
    additional_args="--network anvil-localhost ${additional_args}"
  fi

  log "Additional args: \"${additional_args}\""

  env \
    DEBUG="hopr*,libp2p*" \
    NODE_ENV="development" \
    HOPRD_HEARTBEAT_INTERVAL=2500 \
    HOPRD_HEARTBEAT_THRESHOLD=2500 \
    HOPRD_HEARTBEAT_VARIANCE=1000 \
    HOPRD_NETWORK_QUALITY_THRESHOLD="0.3" \
    HOPRD_ON_CHAIN_CONFIRMATIONS=2 \
    node --experimental-wasm-modules packages/hoprd/lib/main.cjs \
      --announce \
      --api-token "^^LOCAL-testing-123^^" \
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

#  --- Run nodes --- {{{
setup_node 13301 19091 18081 "${node1_dir}" "${node1_dir}.log" "/tmp/${node_prefix}-1" "127.0.0.1"
