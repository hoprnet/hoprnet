#!/usr/bin/env bash

cd /opt/hopr/ || exit 1

yarn && yarn build

# Start the HardHat network on localhost
echo "Starting HardHat network..."
HOPR_ENVIRONMENT_ID=hardhat-localhost yarn run:network > /dev/null 2>&1 &

while [[ "$(curl -s -o /dev/null -w ''%{http_code}'' 127.0.0.1:8545)" != "200" ]]; do
	echo "Waiting for hardhat network to come up..."
	sleep 5;
done

echo "HardHat provider started up!"
cp -R packages/ethereum/deployments/hardhat-localhost/localhost/* packages/ethereum/deployments/hardhat-localhost/hardhat

# Mirror localhost:8545 -> outboud 8546
socat TCP-LISTEN:8546,fork TCP:127.0.0.1:8545 &

echo "Done."