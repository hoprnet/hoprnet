#!/usr/bin/env bash

cd /opt/hopr || exit 1

foundry-tool --environment-name anvil-localhost --environment-type development \
	faucet --password switzerland --use-local-identities \
	--identity-prefix "local" --identity-directory "/var/hopr/identities/" \
	--private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
	--make-root "./packages/ethereum/contracts"
