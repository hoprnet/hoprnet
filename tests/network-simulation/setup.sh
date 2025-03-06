#!/usr/bin/env bash

set -Eeuo pipefail

: "${HOPRD_IMAGE:?"env var HOPRD_IMAGE is required"}"

let hoprd_count=3

# disallow bridge traffic to go through ip tables chain
# See: https://unix.stackexchange.com/questions/499756/how-does-iptable-work-with-linux-bridge
# and: https://wiki.libvirt.org/page/Net.bridge.bridge-nf-call_and_sysctl.conf
echo "Disable bridge-nf-*..."
pushd /proc/sys/net/bridge
for f in bridge-nf-*; do echo 0 > $f; done
popd
echo "Disable bridge-nf-*... Done."

echo "Add network namespaces runtime directory (if not exists)..."
sudo mkdir -p /var/run/netns
echo "Add network namespaces runtime directory... Done."

echo "Start containers..."
docker compose -f compose.yaml up -d
echo "Start containers... Done."

for i in $(seq 1 $hoprd_count); do
	echo "Set up networking for hoprd-$i..."

	# Add bridge interface
	sudo ip link add name ns-br-$i type bridge

	# Add tap device
	sudo ip tuntap add ns-tap-$i mode tap
	sudo ifconfig ns-tap-$i 0.0.0.0 promisc up

	# Add tap device to bridge and activate
	sudo ip link set ns-tap-$i master ns-br-$i
	sudo ip link set ns-br-$i up

	# Get container PID
	let pid=0
	pid=$(docker inspect --format '{{ .State.Pid }}' hopr-network-simulation-hoprd-$i)

	# Soft-link the network namespace created by container into the linux namespace runtime
	sudo ln -s /proc/$pid/ns/net /var/run/netns/$pid

	# Create Veth pair
	sudo ip link add ns-internal-$i type veth peer name ns-external-$i
	sudo ip link set ns-internal-$i master ns-br-$i
	sudo ip link set ns-internal-$i up

	# Configure container-side pair with an interface and address
	sudo ip link set ns-external-$i netns $pid
	sudo ip netns exec $pid ip link set dev ns-external-$i name eth0
	sudo ip netns exec $pid ip link set eth0 address 1$i:34:88:5D:61:BD
	sudo ip netns exec $pid ip link set eth0 up
	sudo ip netns exec $pid ip addr add 10.0.0.$i/16 dev eth0

	echo "Set up networking for hoprd-$i... Done."
done

echo "### Setup complete. Ready to start simulation ###"
