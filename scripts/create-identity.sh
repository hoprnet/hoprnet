#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail
#set -x

# set log id and use shared log function for readable logs
declare script_dir
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare workspace_dir
workspace_dir="${script_dir}/.."


: "${DEPLOYER_PRIVATE_KEY?"Missing environment variable DEPLOYER_PRIVATE_KEY"}"
: "${IDENTITY_PASSWORD?"Missing environment variable IDENTITY_PASSWORD"}"
export PRIVATE_KEY=${DEPLOYER_PRIVATE_KEY}

if ! command -v hopli; then
    export PATH="${PATH}:${workspace_dir}/packages/hopli/.cargo/bin"
fi

network=${1:-rotsee}
number=${2:-1}
docker_tag=${3:-latest}
api_token=${4:-^FOUR2viasj292981FJFKSOAAmnba^}
random="${RANDOM}"

for (( i=1; i<=${number}; i++ )) do
    echo "Generating identity $i"
    declare identity_directory
    identity_directory="${workspace_dir}/dist/identity-${random}/$i"
    mkdir -p ${identity_directory}

    hopli identity --action create --identity-directory "${identity_directory}" --identity-prefix .hoprd
    mv "${identity_directory}/.hoprd0.id" "${identity_directory}/.hoprd.id"
    hopli identity --action read --identity-directory "${identity_directory}" | jq -r ' .[0]' | tee "${identity_directory}/hoprd.json"

    echo "Generate safes"
    hopli create-safe-module --network "${network}" --contracts-root "${workspace_dir}/packages/ethereum/contracts" --identity-from-path "${identity_directory}/.hoprd.id" --hopr-amount 20 --native-amount 0.1 | tee "${identity_directory}/safe.log"

    declare -a safe_address
    safe_address=$(grep "Logs" -A 3 "${identity_directory}/safe.log" | grep safeAddress | cut -d ' ' -f 4)

    declare -a module_address
    module_address=$(grep "Logs" -A 3 "${identity_directory}/safe.log" | grep safeAddress | cut -d ' ' -f 6)

    cat <<EOF > "${identity_directory}/.env"
HOPRD_NETWORK=${network}
HOPRD_DOCKER_IMAGE=europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:${docker_tag}
HOPRD_PASSWORD=${IDENTITY_PASSWORD}
HOPRD_API_TOKEN=${api_token}
HOPRD_SAFE_ADDRESS=${safe_address}
HOPRD_MODULE_ADDRESS=${module_address}
EOF

    cat <<< $(jq --arg safe_address ${safe_address} --arg module_address ${module_address} ' . |= .+ { "safe_address": $safe_address, "module_address": $module_address}' "${identity_directory}/hoprd.json") > "${identity_directory}/hoprd.json"

done

