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

get_hopr_address(){
  echo $(curl $1/api/v1/address/hopr)
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

# Validate Eth addr 1
ETH_ADDRESS1="$(get_eth_address $API1)"
if [ -z "$ETH_ADDRESS1" ]; then
  echo "missing ETH_ADDRESS1"
  exit 1
fi
IS_VALID_ETH_ADDRESS1=$(node -e \
  "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS1'))")
if [ "$IS_VALID_ETH_ADDRESS1" == "false" ]; then
  echo "⛔️ Node outputs an invalid address: $ETH_ADDRESS1"
  exit 1
fi

ETH_ADDRESS2="$(get_eth_address $API2)"
if [ -z "$ETH_ADDRESS2" ]; then
  echo "missing ETH_ADDRESS2"
  exit 1
fi

ETH_ADDRESS3="$(get_eth_address $API3)"
if [ -z "$ETH_ADDRESS3" ]; then
  echo "missing ETH_ADDRESS3"
  exit 1
fi

echo "- Query node-1"
BALANCE="$(run_command $API1 'balance')"
ETH_BALANCE=$(echo -e "$BALANCE" | grep -c " xDAI" || true)
HOPR_BALANCE=$(echo -e "$BALANCE" | grep -c " HOPR" || true)
if [[ "$ETH_BALANCE" != "0" && "$HOPR_BALANCE" != "Hopr Balance: 0 HOPR" ]]; then
  echo "- Node 1 is funded"
else
  echo "⛔️ Node has an invalid balance: $ETH_BALANCE, $HOPR_BALANCE"
  echo -e "$BALANCE"
  exit 1
fi

echo "$(run_command $API1 'peers')"
HOPR_ADDRESS1=$(get_hopr_address $API1)
echo "HOPR_ADDRESS1: $HOPR_ADDRESS1"

HOPR_ADDRESS2=$(get_hopr_address $API2)
echo "HOPR_ADDRESS2: $HOPR_ADDRESS2"

echo "- Node 1 ping node 2: $(run_command $API1 "ping $HOPR_ADDRESS2")"

echo "- Node 1 tickets: $(run_command $API1 'tickets')"

#echo "- Node 1 send 0-hop message to node 2"
#run_command $API1 "send ,$HOPR_ADDRESS2 'hello, world'"

echo "- Node 1 open channel to Node 2"
run_command $API1 "open $HOPR_ADDRESS2 0.1" 

echo "- Node 1 send 1 hop message to self via node 2"
run_command $API1 "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world'"




