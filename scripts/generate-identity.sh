set -eu

: ${HOPR_NETWORK?"must be set"}
: ${IDENTITY_PASSWORD?"must be set"}
: ${PRIVATE_KEY?"must be set"}

contracts_root=${CONTRACTS_ROOT:-"ethereum/contracts"}
export ETHERSCAN_API_KEY=test
export RPC_ENDPOINTS_GNOSIS=https://primary.gnosis-chain.rpc.hoprtech.net

if [ ! -f "${contracts_root}/foundry.toml" ]; then
  echo "Contracts root CONTRACTS_ROOT=${contracts_root} seems to be pointing to the wrong place"
  exit 1
fi

if ! command -v forge >/dev/null; then
  echo "Executable 'forge' not found"
  exit 1
fi

mkdir -p gen

if [ ! -f ./gen/node_0.id ]; then
  ./.cargo/bin/hopli identity \
    --action create \
    --identity-directory "./gen" \
    --identity-prefix node_ \
    --number 1
fi

env DEPLOYER_PRIVATE_KEY="${DEPLOYER_PRIVATE_KEY}" ./.cargo/bin/hopli \
  create-safe-module --network $HOPR_NETWORK \
  --identity-directory "./gen" \
  --hopr-amount 10 --native-amount 0.1 \
  --contracts-root "${contracts_root}"

cp gen/node_0.id hoprd.id
