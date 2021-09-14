#!/usr/bin/env bash

set -e #u
shopt -s expand_aliases
#set -o xtrace

source scripts/environments.sh
source scripts/testnet.sh
source scripts/cleanup.sh

# ---- On Deployment -----
#
# This is run on pushes to master, or release/**
#
# ENV Variables:
# - GITHUB_REF: ie. `/refs/heads/mybranch`
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

# Get version from package.json if not already set
if [ -z "${RELEASE:-}" ]; then
  RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")
fi

_jq() {
  echo "$1" | base64 --decode | jq -r "$2"
}

echo "Looking for releases to deploy (GITHUB_REF == ${GITHUB_REF})"

# iterate through releases with git_ref == $GITHUB_REF
for row in $(cat packages/hoprd/releases.json | ./node_modules/.bin/strip-json-comments | jq -r ".[] | select(.git_ref==\"${GITHUB_REF}\") | @base64"); do
  declare RELEASE_ID=$(_jq "${row}" ".id")
  declare DEPRECATED=$(_jq "${row}" ".deprecated")
  declare ENVIRONMENT_ID=$(_jq "${row}" ".environment_id")
  declare VERSION_MAJOR=$(_jq "${row}" ".version_major")
  declare VERSION_MINOR=$(_jq "${row}" ".version_minor")
  declare DOCKER_IMAGE=$(_jq "${row}" ".docker_image")

  if [ "${DEPRECATED}" == "true" ]; then
    echo "${RELEASE_ID} deprecated, skipping"
    continue
  fi

  declare VERSION_MAJ_MIN
  if [ "${VERSION_MAJOR}" != "null" ] && [ "${VERSION_MINOR}" != "null" ]; then
    VERSION_MAJ_MIN="${VERSION_MAJOR}.${VERSION_MINOR}"
  else
    VERSION_MAJ_MIN="unversioned"
  fi
  declare TESTNET_NAME="$RELEASE_ID-$(echo "$VERSION_MAJ_MIN" | sed 's/\./-/g')"
  declare TESTNET_SIZE=3

  echo "Deploying release ${RELEASE_ID}"
  echo " version: ${VERSION_MAJ_MIN}" 
  echo " environment ${ENVIRONMENT_ID}"
  echo " docker image: ${DOCKER_IMAGE}"
  echo " testnet name: ${TESTNET_NAME}"

  echo "Cleaning up testnet"
  cleanup_instance "${TESTNET_NAME}"
  echo "Starting testnet"
  start_testnet $TESTNET_NAME $TESTNET_SIZE $DOCKER_IMAGE $ENVIRONMENT_ID
done
