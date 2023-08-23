#!/usr/bin/env bash

set -x

WORKDIR=/app/hoprnet
NODE_DIR="${WORKDIR}/node"
CONTRACTS_DIR="${WORKDIR}/packages/ethereum/contracts"


: "${PRIVATE_KEY?"Missing environment variable PRIVATE_KEY"}"
: "${DEPLOYER_PRIVATE_KEY?"Missing environment variable DEPLOYER_PRIVATE_KEY"}"
: "${IDENTITY_PASSWORD?"Missing environment variable IDENTITY_PASSWORD"}"


echo "Generate identity"
hopli identity --action create --identity-directory "${NODE_DIR}" --identity-prefix .hoprd
mv "${NODE_DIR}/.hoprd0.id" "${NODE_DIR}/.hoprd.id"
hopli identity --action read --identity-directory "${NODE_DIR}" | jq -r ' .[0]' > .hoprd.json

echo "Generate safes"
hopli create-safe-module --network "${NETWORK}" --contracts-root "${CONTRACTS_DIR}" --identity-from-path "${NODE_DIR}/.hoprd.id" > safe.log
declare -a safe_address
safe_address=$(grep "Logs" -A 3 safe.log | grep safeAddress | cut -d ' ' -f 4)

declare -a module_address
module_address=$(grep "Logs" -A 3 safe.log | grep safeAddress | cut -d ' ' -f 6)
rm safe.log
cat <<< $(jq --arg safe_address ${safe_address} --arg module_address ${module_address} ' . |= .+ { "safe_address": $safe_address, "module_address": $module_address }' .hoprd.json) > .hoprd.json


