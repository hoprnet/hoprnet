#!/usr/bin/env bash

provider_ip="192.168.101.10"

# Setup socat proxy for anvil if needed
if [ "$(curl -s -o /dev/null -w ''%{http_code}'' localhost:8545)" != "200" ]; then
  while [[ "$(curl -s -o /dev/null -w ''%{http_code}'' $provider_ip:8546)" != "200" ]]; do
    echo "Waiting for the provider at $provider_ip:8546 to come up..."
    sleep 5;
  done

  # Send localhost:8545 requests to 192.168.101.10:8546
  socat TCP-LISTEN:8545,fork TCP:$provider_ip:8546 &
fi
