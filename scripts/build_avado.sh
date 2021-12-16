#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

cd "${mydir}/../packages/avado"

declare AVADO_VERSION="${1}"

# Write AVADO docker build version
sed -i "s/image:[ ]'hopr.avado.dnp.dappnode.eth:[0-9]*\.[0-9]*\.[0-9]*/image: 'hopr.avado.dnp.dappnode.eth:${AVADO_VERSION}/"  ./docker-compose.yml

# Write dappnode version
sed -i "s/\"version\":[ ]\"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"${AVADO_VERSION}\"/"  ./dappnode_package.json

# Must be installed globally due to bad directory calls
npm install -g git+https://github.com/AvadoDServer/AVADOSDK.git#de9f16d

# Must run as sudo due to underlying call to docker-compose
sudo avadosdk build --provider http://80.208.229.228:5001

git add dappnode_package.json docker-compose.yml releases.json
git commit -m "chore(release): publish Avado ${AVADO_VERSION}"
# http://go.ava.do/install/<IPFS HASH>