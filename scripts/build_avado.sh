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
  msg "Usage: $0 [-h|--help] version [environment]"
  msg
  msg "Sets the version of the AVADO build and builds the image. Version must be semver"
  msg "Optional: set default environment"
  msg
}

if [ -z "${1:-}" ]; then
  msg "Missing version"
  usage
  exit 1
fi

declare environment_id="${2:-"$(${mydir}/get-default-environment.sh)"}"

if [ -z "${environment_id}" ]; then
  msg "Could not determine default environment"
  exit 1
fi

# Retrieve the provider URL for the given environment ID from protocol-config.json
provider_url=$(jq -r ".environments.\"${environment_id}\".network_id as \$envid | .networks[\$envid].default_provider // \"\"" "${mydir}/../packages/core/protocol-config.json")
if [ -z "${provider_url}" ]; then
  msg "Environment ${environment_id} has invalid network_id"
  exit 1
fi

if [[ "${1}" == "-h" ]] || [[ "${1}" == "--help" ]]; then
  usage
  exit 0
fi

msg "Building Avado for ${environment_id} with default provider ${provider_url}"

declare AVADO_VERSION="${1}"

if ! [[ $AVADO_VERSION =~ [0-9]{1,}\.[0-9]{1,}\.[0-9]{1,}$ ]]; then
  # AVADO build tool requires proper semver versions
  msg "Version '${AVADO_VERSION}' is not semver"
  exit 1
fi

cd "${mydir}/../packages/avado"

declare default_development_environment="master-goerli"

if [[ -z $(grep -E "${default_development_environment}" "./build/Dockerfile") ]]; then
  # Fail if default environment is no longer `master-goerli`
  msg "Avado Dockerfile differs. Could not find \"${default_development_environment}\" environment in it"
  exit 1
fi

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

# Create backups
cp ./docker-compose.yml ./docker-compose.bak
cp ./dappnode_package.json ./dappnode_package.bak
cp ./build/Dockerfile ./build/Dockerfile.bak

trap cleanup SIGINT SIGTERM ERR EXIT

# Write AVADO docker build version & used provider
sed -E "s/image:[ ]'hopr\.avado\.dnp\.dappnode\.eth:[0-9]\{1,\}\.[0-9]\{1,\}\.[0-9]\{1,\}/image: 'hopr.avado.dnp.dappnode.eth:${AVADO_VERSION}/ ;\
 s/(.+HOPRD_ENVIRONMENT=).+'(,?)/\1${environment_id}'\2/ ; s|(.+HOPRD_PROVIDER=).+'(,?)|\1${provider_url}'\2|" ./docker-compose.yml \
  > ./docker-compose.yml.tmp && mv ./docker-compose.yml.tmp ./docker-compose.yml

# Copy sections between *_JSON_EXPORT of docker-compose.yaml to dappnode_package.json
# and also set dappnode version
sed -n '/BEGIN_JSON_EXPORT/,/END_JSON_EXPORT/{//!p}' ./docker-compose.yml \
  | sed -E "s/]/],/ ; s/'/\"/g ; s/#([{}])/\1/" \
  | jq -s ".[0].image += .[1] | .[0] | .version = \"${AVADO_VERSION}\"" ./dappnode_package.json /dev/stdin \
  > ./dappnode_package.json.tmp && mv ./dappnode_package.json.tmp ./dappnode_package.json

# Replace API token in the json manifest
declare api_token=$(sed -rn "s/.+HOPRD_API_TOKEN=(.+)',/\1/p" ./docker-compose.yml)
sed -E "s/%TOKEN%/${api_token}/" ./dappnode_package.json  > ./dappnode_package.json.tmp && mv ./dappnode_package.json.tmp ./dappnode_package.json

# Overwrite default environment in Dockerfile with the one currently used
sed -e "s/${default_development_environment}/${environment_id}/" ./build/Dockerfile \
  > ./build/Dockerfile.tmp && mv ./build/Dockerfile.tmp ./build/Dockerfile

# AVADO SDK does not do proper releases, therefore using GitHub + git commit hashes
declare AVADO_SDK_COMMIT="7b035be"

# Must be installed globally due to bad directory calls
npm install -g git+https://github.com/AvadoDServer/AVADOSDK.git#${AVADO_SDK_COMMIT}

# Must run as sudo due to underlying call to docker-compose
sudo avadosdk build --provider http://80.208.229.228:5001

# http://go.ava.do/install/<IPFS HASH>

