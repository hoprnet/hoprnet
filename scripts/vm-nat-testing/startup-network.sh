#!/usr/bin/env bash

declare hopr_dir="/opt/hopr"

cd ${hopr_dir} || exit 1
source "${hopr_dir}/scripts/utils.sh"

declare anvil_rpc_log="/tmp/anvil.logs"

if [ "$(curl -s -o /dev/null -w ''%{http_code}'' 127.0.0.1:8545)" != "200" ]; then
	# make sure other node instances are killed
	sudo pkill node || :
	# Start the HardHat network on localhost
	echo "Starting HardHat network..."
	make -C "${hopr_dir}" run-anvil > ${anvil_rpc_log} 2>&1 &
fi

while [[
	"$(curl -s -o /dev/null -w ''%{http_code}'' 127.0.0.1:8545)" != "200" ||
	! -f "${anvil_rpc_log}" ||
  -z "$(grep "Started HTTP and WebSocket JSON-RPC server" "${anvil_rpc_log}" || echo "")"
	]] ; do
	echo "Waiting for anvil network to come up..."
	sleep 5;
done

echo "HardHat provider started up!"
# Copies all the deployment files including the .chainId file
declare protocol_config="${mydir}/../packages/core/protocol-config.json"
declare deployments_summary="${mydir}/../packages/ethereum/contracts/contracts-addresses.json"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost"
update_protocol_config_addresses "${protocol_config}" "${deployments_summary}" "anvil-localhost" "anvil-localhost2"

if [ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8546)" != "200" ]; then
	# Mirror localhost:8545 -> outboud 8546
	socat TCP-LISTEN:8546,fork TCP:127.0.0.1:8545 &
fi

echo "Done."