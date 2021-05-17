# -- Integration test --
# We assume the existence of a test network with three nodes:
# NODE1, IP1, ETH_ADDRESS1 etc.
#
#
source scripts/testnet.sh

if [ -z "$NODE1" ]; then
  echo "missing NODE1"
  exit 1
fi
if [ -z "$NODE2" ]; then
  echo "missing NODE2"
  exit 1
fi
if [ -z "$NODE3" ]; then
  echo "missing NODE3"
  exit 1
fi
if [ -z "$IP1" ]; then
  echo "missing IP1"
  exit 1
fi
if [ -z "$IP2" ]; then
  echo "missing IP2"
  exit 1
fi
if [ -z "$IP3" ]; then
  echo "missing IP3"
  exit 1
fi
if [ -z "$ETH_ADDRESS1" ]; then
  echo "missing ETH_ADDRESS1"
  exit 1
fi
if [ -z "$ETH_ADDRESS2" ]; then
  echo "missing ETH_ADDRESS2"
  exit 1
fi
if [ -z "$ETH_ADDRESS3" ]; then
  echo "missing ETH_ADDRESS3"
  exit 1
fi

echo "Node 1: $NODE1 IP: $IP1, ETH: $ETH_ADDRESS1"
echo "Node 2: $NODE2 IP: $IP2, ETH: $ETH_ADDRESS2"
echo "Node 3: $NODE3 IP: $IP3, ETH: $ETH_ADDRESS3"

echo "- Query node-1"
echo "$(run_command $IP1 'balance')"
echo "$(run_command $IP1 'peers')"
HOPR_ADDRESS1=$(run_command $IP1 'address')
echo "HOPR_ADDRESS1: $HOPR_ADDRESS1"

echo "- Query node-2"
echo "$(run_command $IP2 'balance')"
echo "$(run_command $IP2 'peers')"
HOPR_ADDRESS2=$(run_command $IP2 'address')
echo "HOPR_ADDRESS2: $HOPR_ADDRESS2"

echo "- Node 1 ping node 2"
run_command $IP1 "ping $HOPR_ADDRESS2"






