#!/usr/bin/env bash

set -e #u
shopt -s expand_aliases
#set -o xtrace

set -x

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# ---- On Deployment -----
#
# This finds matching entries in packages/hoprd/releases.json and deploys accordingly
#
# ENV Variables:
# - GITHUB_REF: ie. `/refs/heads/mybranch`
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

_jq() {
  echo "$1" | base64 --decode | jq -r "$2"
}

echo "Looking for releases to deploy (GITHUB_REF == ${GITHUB_REF})"

# iterate through releases with git_ref == $GITHUB_REF
for row in $(cat packages/hoprd/releases.json | jq -r ".[] | select(.git_ref==\"${GITHUB_REF}\") | @base64"); do
  declare release_id=$(_jq "${row}" ".id")
  declare deprecated=$(_jq "${row}" ".deprecated")
  declare environment_id=$(_jq "${row}" ".environment_id")
  declare version_major=$(_jq "${row}" ".version_major")
  declare version_minor=$(_jq "${row}" ".version_minor")
  declare docker_image=$(_jq "${row}" ".docker_image")

  if [ "${deprecated}" == "true" ]; then
    echo "${release_id} deprecated, skipping"
    continue
  fi

  declare version_maj_min
  if [ "${version_major}" != "null" ] && [ "${version_minor}" != "null" ]; then
    version_maj_min="${version_major}.${version_minor}"
  else
    version_maj_min="unversioned"
  fi
  declare testnet_name="$release_id-$(echo "$version_maj_min" | sed 's/\./-/g')"
  declare testnet_size=3

  echo "Deploying release ${release_id}"
  echo " version: ${version_maj_min}"
  echo " environment ${environment_id}"
  echo " docker image: ${docker_image}"
  echo " testnet name: ${testnet_name}"

  echo "Cleaning up testnet"
  cleanup_instance "${testnet_name}"
  echo "Starting testnet"
  start_testnet $testnet_name $testnet_size $docker_image $environment_id
done
