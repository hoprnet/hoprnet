#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# remove previously generated docs to ensure renamed/removed modules are not kept in the docs
dir=$(dirname $(readlink -f $0))
rm -rf "${dir}/../packages/*/docs"

yarn build
yarn docs:generate
