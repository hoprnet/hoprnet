# -- Integration test --
# We assume the existence of a test network with three nodes:
# NODE1, IP1, ETH_ADDRESS1 etc.
#
#
source scripts/testnet.sh

echo "Node 1: $NODE1 IP: $IP1, ETH: $ETH_ADDRESS1"
echo "Node 2: $NODE2 IP: $IP2, ETH: $ETH_ADDRESS2"
echo "Node 3: $NODE3 IP: $IP3, ETH: $ETH_ADDRESS3"

echo "Query node-1"
echo "$(run_command $IP1 'balance')"
echo "$(run_command $IP1 'peers')"







#echo "Cleaning up after deploy"
#cleanup
