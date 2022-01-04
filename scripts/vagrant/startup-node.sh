#!/usr/bin/env bash

provider_ip="192.168.101.10"
id_dir="/var/hopr/identities"

mkdir -p $id_dir

rnd_name=$(tr -cd 'a-f0-9' < /dev/urandom | head -c 8)
id_file="$id_dir/local-$rnd_name.id"
echo "Node identity is in $id_file"

# Setup socat proxy for hardhat
if [ "$(curl -s -o /dev/null -w ''%{http_code}'' localhost:8545)" = "200" ]; then
  while [[ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8546)" != "200" ]]; do
  	echo "Waiting for the provider at $provider_ip:8546 to come up..."
  	sleep 5;
  done
fi

# Send localhost:8545 requests to 192.168.101.10:8546
socat TCP-LISTEN:8545,fork TCP:$provider_ip:8546 &

DEBUG="hopr*" node /opt/hopr/packages/hoprd/lib/index.js --identity "$id_file" "$@"
