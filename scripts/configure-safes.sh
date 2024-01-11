#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare input_file input_len

: "${DEPLOYER_PRIVATE_KEY?"Missing environment variable DEPLOYER_PRIVATE_KEY"}"
: "${1:?"1st parameter <nodes_info> missing"}"
[ -f "${1}" ] || { echo "1st parameters <node_info> does not point to a file"; exit 1; }

input_file="${1}"
input_len="$(jq '.identities | length' ${input_file})"

for i in $(seq 0 $((${input_len}-1))); do
	declare node_addr safe_addr module_addr
	safe_addr="$(jq -r ".identities[$i].safe_address" ${input_file})"
	module_addr="$(jq -r ".identities[$i].module_address" ${input_file})"
	node_addr="$(jq -r ".identities[$i].\".hoprd.id\".native_address" ${input_file})"

	echo "Node nr: ${i}"
	echo "Safe: ${safe_addr}"
	echo "Module: ${module_addr}"
	echo "Node: ${node_addr}"

	env PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" make -C ethereum/contracts \
		configure-safe-module network=rotsee environment-type=staging \
		node_address="${node_addr}" \
		safe_address="${safe_addr}" \
		module_address="${module_addr}"
done


