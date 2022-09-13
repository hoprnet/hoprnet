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
  msg "Usage: $0 [-h|--help] version [environment] [release] [api token]"
  msg
  msg "Sets the version of the AVADO build and builds the image. Version must be semver"
  msg "Optional: set default environment, release name and API token"
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

# Get/set default values of parameters
declare avado_version="${1}"
declare environment_id="${2:-"$(${mydir}/get-default-environment.sh)"}"
declare release_id="${3:-"$(${mydir}/get-default-environment.sh --release)"}"
declare api_token="${4:-"!5qxc9Lp1BE7IFQ-nrtttU"}" # <- Default AVADO API token

# Validate environment and release ids
if [[ -z "${environment_id}" ]] || [[ -z "${release_id}" ]]; then
  msg "Could not determine default environment or release id"
  exit 1
fi

# AVADO build tool requires proper semver versions
if ! [[ $avado_version =~ [0-9]{1,}\.[0-9]{1,}\.[0-9]{1,}$ ]]; then
  msg "Version '${avado_version}' is not semver"
  exit 1
fi

# Retrieve the provider URL for the given environment ID from protocol-config.json
declare provider_url=$(jq -r ".environments.\"${environment_id}\".network_id as \$envid | .networks[\$envid].default_provider // \"\"" "${mydir}/../packages/core/protocol-config.json")
if [ -z "${provider_url}" ]; then
  msg "Environment ${environment_id} has invalid network_id"
  exit 1
fi


cd "${mydir}/../packages/avado"

function cleanup {
  local EC=$?
  trap - SIGINT SIGTERM ERR EXIT
  set +Eeuo pipefail

  # Undo changes restoring backups
  mv ./docker-compose.bak ./docker-compose.yml
  mv ./dappnode_package.bak ./dappnode_package.json
  mv ./build/Dockerfile.bak ./build/Dockerfile

  exit $EC
}

msg "Building Avado v. ${avado_version} for release ${release_id} using environment ${environment_id} with default provider ${provider_url}"

# Create backups
cp ./docker-compose.yml ./docker-compose.bak
cp ./dappnode_package.json ./dappnode_package.bak
cp ./build/Dockerfile ./build/Dockerfile.bak

trap cleanup SIGINT SIGTERM ERR EXIT

### Update docker-compose.yaml
sed -E "s/%AVADO_VERSION%/${avado_version}/g ; s/%TOKEN%/${api_token}/g ; \
 s/%ENV_ID%/${environment_id}/g ; s|%PROVIDER_URL%|${provider_url}|g" ./docker-compose.yml \
  > ./docker-compose.yml.tmp && mv ./docker-compose.yml.tmp ./docker-compose.yml

###
### Update dappnode_package.json

# Copy sections between *_JSON_EXPORT of docker-compose.yaml to dappnode_package.json
# and also set dappnode version
sed -n '/BEGIN_JSON_EXPORT/,/END_JSON_EXPORT/{//!p}' ./docker-compose.yml \
  | sed -E "s/]/],/ ; s/'/\"/g ; s/#([{}])/\1/" \
  | jq -s ".[0].image += .[1] | .[0] | .version = \"${avado_version}\"" ./dappnode_package.json /dev/stdin \
  > ./dappnode_package.json.tmp && mv ./dappnode_package.json.tmp ./dappnode_package.json

sed -E "s/%TOKEN%/${api_token}/g" ./dappnode_package.json \
  > ./dappnode_package.json.tmp && mv ./dappnode_package.json.tmp ./dappnode_package.json

###

# AVADO SDK does not do proper releases, therefore using GitHub + git commit hashes
declare avado_sdk_commit="7b035be"

# Must be installed globally due to bad directory calls
npm install -g git+https://github.com/AvadoDServer/AVADOSDK.git#${avado_sdk_commit}

# Must run as sudo due to underlying call to docker-compose
sudo avadosdk build --provider http://80.208.229.228:5001

# http://go.ava.do/install/<IPFS HASH>
