#!/usr/bin/env bash

cd /opt/hopr/ && yarn && yarn build

DEBUG="hopr*" node packages/hoprd/lib/index.js --admin --adminHost 0.0.0.0 --healthCheck --healthCheckHost 0.0.0.0 --init --rest --restHost 0.0.0.0 --environment hardhat-localhost --apiToken MyT0ken123^ --password switzerland --testPreferLocalAddresses --identity /var/hopr/identities/local-public.id --announce
