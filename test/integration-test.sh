# Runs a testnet, and tests against it.
# This relies on using GCloud as an environment for testing

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

TESTNET_NAME="integration-test$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
TESTNET_SIZE=3

#echo "Cleaning up before deploy"
#cleanup

#echo "Starting testnet '$TESTNET_NAME' with $TESTNET_SIZE nodes and image hoprd:$RELEASE"
#start_testnet $TESTNET_NAME $TESTNET_SIZE "gcr.io/hoprassociation/hoprd:$RELEASE" 

echo "Running integration test against testnet: $TESTNET_NAME"

NODE="$(vm_name 'node-1' $TESTNET_NAME)"
echo "Getting address for $NODE"
IP="$(gcloud_get_ip $NODE)"
echo "IP: $IP"
ETH_ADDRESS="$(get_eth_address $IP)"

# TODO FUND ADDRESS

echo "ETH: $ETH_ADDRESS"
echo "$(run_command $IP 'balance')"
echo "$(run_command $IP 'peers')"







#echo "Cleaning up after deploy"
#cleanup
