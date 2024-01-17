#!/usr/bin/env bash

cd /opt/hopr || exit 1

IDENTITY_PASSWORD=switzerland PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
	hopli faucet \
	--environment-name anvil-localhost \
	--identity-prefix "local" --identity-directory "/var/hopr/identities/" \
	--contracts-root "./ethereum/contracts"
