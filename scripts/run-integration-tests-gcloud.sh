#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && (echo "This script should only be executed."; exit 1)

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="e2e-test-run-gcloud"
source "$(dirname $(readlink -f $0))/utils.sh"

# we need the testnet helper functions
source "$(dirname $(readlink -f $0))/utils.sh"

declare hoprd_docker_image="gcr.io/hoprassociation/hoprd:${HOPR_TESTNET_VERSION}"
declare testnet_size="${HOPR_TESTNET_SIZE:3}"

testnet_start "${HOPR_TESTNET_NAME}" "${testnet_size}" "${hoprd_docker_image}"

testnet_destroy "${HOPR_TESTNET_NAME}" "${testnet_size}"
