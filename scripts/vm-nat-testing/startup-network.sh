#!/usr/bin/env bash

declare hopr_dir="/opt/hopr"

cd ${hopr_dir} || exit 1

declare hardhat_rpc_log="/tmp/hardhat.logs"

if [ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8545)" != "200" ]; then
	# make sure other instances are killed
	sudo pkill node || :
	# Start the HardHat network on localhost
	echo "Starting HardHat network..."
	TS_NODE_PROJECT=${hopr_dir}/packages/ethereum/tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID=hardhat-localhost \
	DEVELOPMENT=true \
	yarn workspace @hoprnet/hopr-ethereum hardhat node > ${hardhat_rpc_log} 2>&1 &
fi

while [[
	"$(curl -s -o /dev/null -w ''%{http_code}'' 127.0.0.1:8545)" != "200" ||
	! -f "${hardhat_rpc_log}" ||
  -z "$(grep "Started HTTP and WebSocket JSON-RPC server" "${hardhat_rpc_log}" || echo "")"
	]] ; do
	echo "Waiting for hardhat network to come up..."
	sleep 5;
done

echo "HardHat provider started up!"
cp -R packages/ethereum/deployments/hardhat-localhost/localhost/* packages/ethereum/deployments/hardhat-localhost/hardhat

if [ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8546)" != "200" ]; then
	# Mirror localhost:8545 -> outboud 8546
	socat TCP-LISTEN:8546,fork TCP:127.0.0.1:8545 &
fi

echo "Done."