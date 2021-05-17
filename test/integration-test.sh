# -- Integration test --
# We assume the existence of a test network with three nodes:
# API1, API2 API3.
#
#
# $1 = IP
# $2 = Hopr command
run_command(){
  curl --silent -X POST --data "$2" $1/api/v1/command
}
get_eth_address(){
  echo $(curl $1/api/v1/address/eth)
}

if [ -z "$API1" ]; then
  echo "missing API1"
  exit 1
fi
if [ -z "$API2" ]; then
  echo "missing API2"
  exit 1
fi
if [ -z "$API3" ]; then
  echo "missing API3"
  exit 1
fi

echo "Node 1: $API1"
echo "Node 2: $API2"
echo "Node 3: $API3"

ETH_ADDRESS1="$(get_eth_address $API1)"
ETH_ADDRESS2="$(get_eth_address $API2)"
ETH_ADDRESS3="$(get_eth_address $API3)"

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

echo "- Query node-1"
echo "$(run_command $API1 'balance')"
echo "$(run_command $API1 'peers')"
HOPR_ADDRESS1=$(run_command $API1 'address')
echo "HOPR_ADDRESS1: $HOPR_ADDRESS1"

IS_VALID_ETH_ADDRESS=$(node -e \
  "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS'))")
if [ "$IS_VALID_ETH_ADDRESS" == "true" ]; then
  echo "✅ Node outputs a valid address: $IS_VALID_ETH_ADDRESS"
  exit 0
else
  echo "⛔️ Node outputs an invalid address: $ETH_ADDRESS"
  exit 1
fi

HOPR_ADDRESS2=$(run_command $API2 'address')
echo "HOPR_ADDRESS2: $HOPR_ADDRESS2"

echo "- Node 1 ping node 2"
run_command $API1 "ping $HOPR_ADDRESS2"

echo "- Node 1 tickets"
run_command $API1 "tickets"

echo "- Node 1 send 0-hop message to node 2"
run_command $API1 "send ,$HOPR_ADDRESS2 'hello, world'"






