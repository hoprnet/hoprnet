#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_PACKAGE:-}" && (echo "Missing environment variable HOPR_PACKAGE"; exit 1)
test -z "${HOPR_PACKAGE_VERSION:-}" && (echo "Missing environment variable HOPR_PACKAGE_VERSION"; exit 1)

npm view "@hoprnet/${HOPR_PACKAGE:-}@${HOPR_PACKAGE_VERSION:-}" --json
