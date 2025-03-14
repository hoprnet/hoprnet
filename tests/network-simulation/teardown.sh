#!/usr/bin/env bash

set -Eeuo pipefail

let hoprd_count=3

for i in $(seq 1 $hoprd_count); do
  echo "Disable and delete bridge and tap interfaces for hoprd-$i..."
  sudo ip link set ns-br-$i down
  sudo ip link set ns-tap-$i nomaster
  sudo ip link del ns-br-$i
  sudo ip link del ns-tap-$i
  echo "Disable and delete bridge and tap interfaces for hoprd-$i... Done"
done

echo "Stop all running container..."
docker compose -f compose.yaml down
echo "Stop all running container... Done"
