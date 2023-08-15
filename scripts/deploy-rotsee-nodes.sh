#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC1090
source "${mydir}/testnet.sh"

declare branch docker_image

docker_image="${1:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:latest}"
branch=${2:-master}
: "${3:?"3rd parameter <ssh_hosts> missing"}"
ssh_hosts=( ${3} )

NODE_NAME=hoprd-node-rotsee-providence
API_TOKEN=^binary6wire6GLEEMAN9urbanebetween1watch^
IDENTITY_PASSWORD=some2TABLE5lamp1glas9WHITE8wood
SSH_USER=root
NETWORK=rotsee

: "${DEPLOYER_PRIVATE_KEY?"Missing environment variable DEPLOYER_PRIVATE_KEY"}"
declare -a hopr_addrs

run_node() {
  local host safe_args
  host="${1}"
  safe_args="${2}"

  echo ""
  echo "==="
  echo "Running node on host ${host}"
  echo "==="
  echo ""

  ssh -tt ${SSH_USER}@${host} <<EOF
  	docker stop ${NODE_NAME} || echo 'HOPRd node was not running'
    docker rm ${NODE_NAME} || echo 'HOPRd node was not created'
		docker run -d --pull always --restart on-failure -m 4g \
		  --name "${NODE_NAME}" \
		  --log-driver json-file --log-opt max-size=1000M --log-opt max-file=5 \
		  -ti -v /root/hoprd-db:/app/hoprd-db \
		  -p 9091:9091/tcp -p 9091:9091/udp -p 8080:8080 -p 3001:3001 -e DEBUG="*" \
      ${docker_image} \
      --environment ${NETWORK} \
      --init --api \
      --announce \
      --identity /app/hoprd-db/.hopr-id \
      --data /app/hoprd-db \
      --password '${IDENTITY_PASSWORD}' \
      --apiHost "0.0.0.0" --apiToken "${API_TOKEN}" \
      --healthCheck --healthCheckHost "0.0.0.0" \
      --heartbeatInterval 20000 --heartbeatThreshold 60000 \
      ${safe_args}
EOF
}

test_ssh_connection() {
  local host="${1}"

  echo ""
  echo "==="
  echo "Testing ssh connection to ${host}"
  echo "==="
  echo ""

  ssh -tt ${SSH_USER}@${host} hostname
}

update_and_restart_host() {
  local host="${1}"

  echo ""
  echo "==="
  echo "Updating and restarting host ${host}"
  echo "==="
  echo ""

  ! ssh -tt ${SSH_USER}@${host} <<EOF
    apt-get update -y
    apt-get upgrade -y
    apt-get dist-upgrade -y
    shutdown -r now
EOF
}

register_nodes() {
  echo ""
  echo "==="
  echo "Register all nodes"
  echo "==="
  echo ""

  IFS=','

  # use CI wallet to register VM instances. This action may fail if nodes were previously linked to other staking accounts
  PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" make -C "${makefile_path}" self-register-node \
    network="${NETWORK}" \
    peer_ids="${hopr_addrs[*]}" \
    environment_type=production
  unset IFS
}

generate_local_identities() {
  echo ""
  echo "==="
  echo "Generate local identities"
  echo "==="
  echo ""

  rm -rf "${mydir}/tmp/"
  mkdir -p "${mydir}/tmp"

  env ETHERSCAN_API_KEY="" IDENTITY_PASSWORD="${IDENTITY_PASSWORD}" \
    ${mydir}/../.cargo/bin/hopli identity \
    --action create \
    --identity-directory "${mydir}/tmp/" \
    --identity-prefix ".hopr-id_" \
    --number "${#ssh_hosts[@]}"
}

upload_identities() {
  echo ""
  echo "==="
  echo "Upload identities"
  echo "==="
  echo ""

  local dest src identities
  dest="/root/hoprd-db/.hopr-id"
  src="${mydir}/tmp/"
  identities="`find ${src} -type f -name '.hopr-id_*'`"

  for i in "${!ssh_hosts[@]}"; do
    local host
    host="${ssh_hosts[$i]}"

    test_ssh_connection "${host}"
    # only upload new identity if none is present
    rsync -av --ignore-existing "${identities[$i]}" "${SSH_USER}@${host}:${dest}"

    # download identity for safe deployment, at this point both files should
    # always be the same
    rsync -av "${SSH_USER}@${host}:${dest}" "${identities[$i]}"
  done
}

deploy_safes() {
  echo ""
  echo "==="
  echo "Deploy safes"
  echo "==="
  echo ""

  local identities
  identities="`find ${src} -type f -name '.hopr-id_*'`"

  for i in "${!identities[@]}"; do
    local id_path
    id_path="${identities[$i]}"

    env \
      ETHERSCAN_API_KEY="" \
      IDENTITY_PASSWORD="${IDENTITY_PASSWORD}" \
      PRIVATE_KEY="DEPLOYER_PRIVATE_KEY" \
      hopli create-safe-module \
      --network "${NETWORK}" \
      --identity-from-path "${id_path}" \
      --contracts-root "./packages/ethereum/contracts" > safe.log

    # store safe arguments in separate file for later use
    grep -oE "(\-\-safeAddress.*)" safe.log > "${id_path}.safe.args"
    rm safe.logs
  done
}

start_nodes() {
  local identities
  identities="`find ${src} -type f -name '.hopr-id_*'`"

  for i in "${!ssh_hosts[@]}"; do
    local host safe_args
    host="${ssh_hosts[$i]}"
    safe_args=$(<"${identities[$i]}.safe.args")

    test_ssh_connection "${host}"
    run_node "${host}" "${safe_args}"
  done
}

### STARTING

for host in ${ssh_hosts[@]}; do
  test_ssh_connection "${host}"
  update_and_restart_host "${host}"
done

generate_local_identities

upload_identities

deploy_safes

start_nodes

for host in ${ssh_hosts[@]}; do
  test_ssh_connection "${host}"

  declare api_wallet_addr
  api_wallet_addr="$(get_native_address "${API_TOKEN}@${host}:3001")"
  #api_peer_id="$(get_hopr_address "${API_TOKEN}@${host}:3001")"
  #hopr_addrs+=( "${api_peer_id}" )

  fund_if_empty "${api_wallet_addr}" "${NETWORK}"
done

# register_nodes
