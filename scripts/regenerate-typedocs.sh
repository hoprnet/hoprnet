#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# remove previously generated docs to ensure renamed/removed modules are not kept in the docs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
rm -rf "${mydir}/../packages/*/docs"

yarn build
yarn docs:generate
