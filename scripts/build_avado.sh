#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [-h|--help] version"
  msg
  msg "Sets the version of the AVADO build and builds the image. Version must be semver"
  msg
}

if [ -z "${1:-}" ]; then
  msg "Missing version"
  usage
  exit 1
fi

if [[ "${1}" == "-h" ]] || [[ "${1}" == "--help" ]]; then
  usage
  exit 0
fi


declare AVADO_VERSION="${1}"

if ! [[ $AVADO_VERSION =~ [0-9]{1,}[.]{1}[0-9]{1,}[.]{1}[0-9]{1,}$ ]]; then
  # AVADO build tool requires proper semver versions
  msg "Version '${AVADO_VERSION}' is not semver"
fi

cd "${mydir}/../packages/avado"

# Write AVADO docker build version
sed -i "s/image:[ ]'hopr.avado.dnp.dappnode.eth:[0-9]*\.[0-9]*\.[0-9]*/image: 'hopr.avado.dnp.dappnode.eth:${AVADO_VERSION}/"  ./docker-compose.yml

# Write dappnode version
sed -i "s/\"version\":[ ]\"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"${AVADO_VERSION}\"/"  ./dappnode_package.json

# AVADO SDK does not do proper releases, therefore using GitHub + git commit hashes
declare AVADO_SDK_COMMIT="de9f16d"

# Must be installed globally due to bad directory calls
npm install -g git+https://github.com/AvadoDServer/AVADOSDK.git#${AVADO_SDK_COMMIT}

# Must run as sudo due to underlying call to docker-compose
sudo avadosdk build --provider http://80.208.229.228:5001

# http://go.ava.do/install/<IPFS HASH>