#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC1091
source "${mydir}/testnet.sh"

declare action docker_image ssh_hosts

: "${1:?"1st parameter <ssh_hosts_file> missing"}"
[ -f "${1}" ] || { echo "1st parameters <ssh_hosts_file> does not point to a file"; exit 1; }
mapfile -t ssh_hosts <<< "$(<${1})"
action="${2:-deploy}"
docker_image="${3:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:latest}"

CONTAINER_NAME=hoprd-node-rotsee-providence
SSH_USER=root
NETWORK=rotsee

PATH="${mydir}/../.foundry/bin:${mydir}/../.cargo/bin:${PATH}"

: "${DEPLOYER_PRIVATE_KEY?"Missing environment variable DEPLOYER_PRIVATE_KEY"}"
: "${API_TOKEN?"Missing environment variable API_TOKEN"}"
: "${IDENTITY_PASSWORD?"Missing environment variable IDENTITY_PASSWORD"}"
declare -a hopr_addrs
declare -a safe_addrs

run_node() {
  local host
  host="${1}"

  echo ""
  echo "==="
  echo "Running node on host ${host}"
  echo "==="
  echo ""

  ssh -o StrictHostKeyChecking=no "${SSH_USER}@${host}" <<EOF
    docker pull ${docker_image} || echo 'Hoprd image not found'
  	docker stop ${CONTAINER_NAME} || echo 'HOPRd node was not running'
    docker rm -f ${CONTAINER_NAME} || echo 'HOPRd node was not created'
    safe_args=\$(</root/hoprd-db/.hopr-id.safe.args)
		docker run -d --pull always --restart on-failure -m 4g \
		  --name "${CONTAINER_NAME}" \
		  --log-driver json-file --log-opt max-size=1000M --log-opt max-file=5 \
		  -ti -v /root/hoprd-db:/app/hoprd-db \
		  -p 9091:9091/tcp -p 9091:9091/udp -p 8080:8080 -p 3001:3001 -e DEBUG="*" \
      ${docker_image} \
      --network ${NETWORK} \
      --init --api \
      --announce \
      --identity /app/hoprd-db/.hopr-id \
      --data /app/hoprd-db \
      --password '${IDENTITY_PASSWORD}' \
      --apiHost "0.0.0.0" --apiToken "${API_TOKEN}" \
      --healthCheck --healthCheckHost "0.0.0.0" \
      --heartbeatInterval 20000 --heartbeatThreshold 60000 \
      \${safe_args}
EOF
}

test_ssh_connection() {
  local host="${1}"

  echo ""
  echo "==="
  echo "Testing ssh connection to ${host}"
  echo "==="
  echo ""

  ssh -o StrictHostKeyChecking=no "${SSH_USER}@${host}" hostname
}

update_and_restart_host() {
  local host="${1}"

  echo ""
  echo "==="
  echo "Updating host ${host}"
  echo "==="
  echo ""

  ! ssh "${SSH_USER}@${host}" <<EOF
    export DEBIAN_FRONTEND="noninteractive"
    apt-get update -y
    apt-get upgrade -y
    apt-get dist-upgrade -y
    apt-get install -y ca-certificates curl gnupg
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg
    echo "deb [arch="\$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian "\$(. /etc/os-release && echo "\$VERSION_CODENAME")" stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt-get update -y
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
EOF

  echo ""
  echo "==="
  echo "Restarting host ${host}"
  echo "==="
  echo ""

  ssh "${SSH_USER}@${host}" "shutdown -r now" || :
}

register_nodes() {
  echo ""
  echo "==="
  echo "Register all nodes"
  echo "==="
  echo ""

  IFS=','

  # use CI wallet to register VM instances. This action may fail if nodes were previously linked to other staking accounts
  PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" make -C "${makefile_path}" register-node \
    network="${NETWORK}" \
    staking_addresses="${safe_addrs[*]}" \
    node_addresses="${hopr_addrs[*]}" \
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
    hopli identity \
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
  mapfile -t identities <<< "$(find "${src}" -type f -name '.hopr-id_*.id' | sort)"

  for i in "${!ssh_hosts[@]}"; do
    local host
    host="${ssh_hosts[$i]}"

    test_ssh_connection "${host}"

    # ensure target folder exists
    ssh "${SSH_USER}@${host}" mkdir -p /root/hoprd-db

    # only upload new identity if none is present
    rsync -av --ignore-existing "${identities[$i]}" "${SSH_USER}@${host}:${dest}"

    # download identity for safe deployment, at this point both files should
    # always be the same
    rsync -av "${SSH_USER}@${host}:${dest}" "${identities[$i]}"

    # download safe args file if it exists, ignore failure if file does not
    # exist
    rsync -av "${SSH_USER}@${host}:${dest}.safe.args" "${identities[$i]}.safe.args" || :
  done
}

deploy_safes() {
  echo ""
  echo "==="
  echo "Deploy safes"
  echo "==="
  echo ""

  local src identities
  src="${mydir}/tmp/"
  mapfile -t identities <<< "$(find "${src}" -type f -name '.hopr-id_*.id' | sort)"

  for i in "${!identities[@]}"; do
    local id_path
    id_path="${identities[$i]}"

    # skip if safe args already exists
    [ -f "${id_path}.safe.args" ] && continue

    env \
      ETHERSCAN_API_KEY="" \
      IDENTITY_PASSWORD="${IDENTITY_PASSWORD}" \
      PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" \
      DEPLOYER_PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" \
      hopli create-safe-module \
      --network "${NETWORK}" \
      --identity-from-path "${id_path}" \
      --contracts-root "./packages/ethereum/contracts" > safe.log

    # store safe arguments in separate file for later use
    grep -oE "(\-\-safeAddress.*)" safe.log > "${id_path}.safe.args"
    rm safe.log

    # upload safe args to host for later use
    rsync -av --ignore-existing "${id_path}.safe.args" "${SSH_USER}@${ssh_hosts[$i]}:/root/hoprd-db/.hopr-id.safe.args"
  done
}

# Deploy latest image hoprd on existing nodes
deploy_nodes() {
  for host in "${ssh_hosts[@]}"; do
    test_ssh_connection "${host}"
    run_node "${host}"
  done
}

### Install nodes from scratch
install_nodes(){
  for host in "${ssh_hosts[@]}"; do
    test_ssh_connection "${host}"
    update_and_restart_host "${host}"
  done

  generate_local_identities

  upload_identities

  # deploy safes, register nodes to NR, approve token transfers
  deploy_safes

  deploy_nodes

  for host in "${ssh_hosts[@]}"; do
    test_ssh_connection "${host}"

    declare api_wallet_addr
    api_wallet_addr="$(get_native_address "${API_TOKEN}@${host}:3001")"
    #api_peer_id="$(get_hopr_address "${API_TOKEN}@${host}:3001")"
    #hopr_addrs+=( "${api_peer_id}" )
    #safe_address=$("cat /root/hoprd-db/.hopr-id.safe.args | grep -oE '\-\-safeAddress [^[:space:]]+' | awk '{print \$2}'")
    #safe_addrs+=( "${safe_address}" )

    fund_if_empty "${api_wallet_addr}" "${NETWORK}"
  done

  # register_nodes

}

case "$action" in
  install)
    # return early with help info when requested
    install_nodes
    ;;
  deploy)
    deploy_nodes
    ;;
esac


