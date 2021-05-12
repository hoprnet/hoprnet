#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_PACKAGE:-}" && (echo "Missing environment variable HOPR_PACKAGE"; exit 1)

# returns the maj-min version as found in the local package info
node -p -e "require('./packages/${HOPR_PACKAGE:-}/package.json').version" | \
  sed 's/\(\.[0-9]*\-alpha\)*\.[0-9]*$//'
