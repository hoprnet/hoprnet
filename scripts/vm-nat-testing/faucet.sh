#!/usr/bin/env bash

cd /opt/hopr || exit 1

HOPR_ENVIRONMENT_ID=hardhat-localhost yarn workspace @hoprnet/hopr-ethereum \
	hardhat faucet --use-local-identities --identity-directory /var/hopr/identities/ --identity-prefix local \
	--password switzerland --network hardhat
