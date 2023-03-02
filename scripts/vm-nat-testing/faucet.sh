#!/usr/bin/env bash

cd /opt/hopr || exit 1

IDENTITY_PASSWORD=switzerland hopli faucet \
	--environment-name anvil-localhost --environment-type development \
	--use-local-identities \
	--identity-prefix "local" --identity-directory "/var/hopr/identities/" \
	--private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
	--contracts-root "./packages/ethereum/contracts"
