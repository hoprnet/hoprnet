#!/usr/bin/env bash

cd /opt/hopr/ || exit 1

declare log_file="/tmp/hardhat.logs"

if [ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8545)" != "200" ]; then
	# Start the HardHat network on localhost
	echo "Starting HardHat network..."
	HOPR_ENVIRONMENT_ID=hardhat-localhost yarn run:network > ${log_file} 2>&1 &
fi

while [[
	"$(curl -s -o /dev/null -w ''%{http_code}'' 127.0.0.1:8545)" != "200" ||
	! -f "${log_file}" ||
  -z $(grep "Started HTTP and WebSocket JSON-RPC server" "${log_file}" || echo "")
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
