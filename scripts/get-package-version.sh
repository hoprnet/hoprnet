#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_PACKAGE:-}" && (echo "Missing environment variable HOPR_PACKAGE"; exit 1)

declare full_version

# get full version info from package description
full_version=$(node -p -e "require('./packages/${HOPR_PACKAGE:-}/package.json').version")

# returns the maj-min version if requested, otherwise the full version
if [ -n "${HOPR_VERSION_MAJMIN:-}" ]; then
  echo "${full_version}" | sed 's/\(\.[0-9]*\-alpha\)*\.[0-9]*$//'
else
  echo "${full_version}"
fi
