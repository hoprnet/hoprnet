#!/bin/bash
set -euo pipefail

errors=""

check_safe() {

  if [ ! -f /etc/hoprd/hoprd.env ]; then
    if [ -z "${HOPRD_SAFE_ADDRESS:-}" ]; then
      errors+="- The 'HOPRD_SAFE_ADDRESS' environment variable is required. You can get it from https://hub.hoprnet.org\n"
    elif ! echo "$HOPRD_SAFE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
      errors+="- Invalid Safe Ethereum address ('HOPRD_SAFE_ADDRESS') format. Please enter a valid address.\n"
    fi

    if [ -z "${HOPRD_MODULE_ADDRESS:-}" ]; then
      errors+="- The 'HOPRD_MODULE_ADDRESS' environment variable is required. You can get it from https://hub.hoprnet.org\n"
    elif ! echo "$HOPRD_MODULE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
      errors+="- Invalid Safe Module Ethereum address ('HOPRD_MODULE_ADDRESS') format. Please enter a valid address.\n"
    fi
  else
    if ! grep -q "^HOPRD_SAFE_ADDRESS=" /etc/hoprd/hoprd.env 2>/dev/null; then
      errors+="- The 'HOPRD_SAFE_ADDRESS' environment variable is required at /etc/hoprd/hoprd.env . You can get it from https://hub.hoprnet.org\n"
    fi

    if ! grep -q "^HOPRD_MODULE_ADDRESS=" /etc/hoprd/hoprd.env 2>/dev/null; then
      errors+="- The 'HOPRD_MODULE_ADDRESS' environment variable is required at /etc/hoprd/hoprd.env. You can get it from https://hub.hoprnet.org\n"
    fi
  fi
}

test_rpc_provider() {
  url=$1
  rpc_response=$(curl -s --connect-timeout 3 --max-time 5 --retry 5 --retry-delay 2 --retry-connrefused -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}' "${url}")
  if ! echo "$rpc_response" | jq -e '.result' >/dev/null; then
    errors+="- The 'HOPRD_PROVIDER' environment variable is not a valid RPC provider URL. Please check the URL and try again.\n"
  fi
}

check_rpc_provider() {

  if [ ! -f "/etc/hoprd/hoprd.env" ]; then
    if [ -z "${HOPRD_PROVIDER:-}" ]; then
      test_rpc_provider "http://localhost:8545"
    else
      test_rpc_provider "${HOPRD_PROVIDER}"
    fi
  else
    if ! grep -q "^HOPRD_PROVIDER=" /etc/hoprd/hoprd.env 2>/dev/null; then
      errors+="- The 'HOPRD_PROVIDER' environment variable is required at /etc/hoprd/hoprd.env. You can get it from https://docs.hoprnet.org/node/custom-rpc-provider\n"
    else
      test_rpc_provider "$(grep "^HOPRD_PROVIDER=" /etc/hoprd/hoprd.env | cut -d'=' -f2-)"
    fi
  fi
}

check_network() {
  # Validate that HOPRD_NETWORK is either "dufour" or "rotsee"
  if [ -n "${HOPRD_NETWORK:-}" ] && [[ ${HOPRD_NETWORK:-} != "dufour" && ${HOPRD_NETWORK:-} != "rotsee" ]]; then
    errors+="- The 'HOPRD_NETWORK' environment variable must be either 'dufour' or 'rotsee'.\n"
  fi
}

check_identity_password() {
  # If the file /etc/hoprd/hopr.id exists then HOPRD_PASSWORD is required
  if [ -f /etc/hoprd/hopr.id ] && [ -z "${HOPRD_PASSWORD:-}" ]; then
    errors+="- There is an existing identity file at /etc/hoprd/hopr.id from previous installation, You have to provide its password via 'HOPRD_PASSWORD' environment variable or delete the identity file.\n"
  fi
}

check_safe
check_rpc_provider
check_network
check_identity_password

if [ -n "${errors}" ]; then
  echo -e "##########################################################################################\n"
  echo -e "################                    HOPRD ERRORS                        ##################\n"
  echo -e "##########################################################################################\n"
  echo -e "The following environment variables are required:\n${errors}"
  echo "Please set them before installing the package."
  echo -e "##########################################################################################\n"

  exit 1
fi
