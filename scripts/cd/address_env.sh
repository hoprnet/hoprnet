#!/bin/bash
# @param {('*-bootstrap')} GCLOUD_VM_NAME - The name of the container
# @param {('*-bootstrap')} GCLOUD_VM_DISK - The disk of the container
# @param {('**')} RELEASE_VERSION - The version of the docker image
# @param {('**')} BS_PASSWORD - The password of the bootstrap server
# @requires gcloud
set -eu
[[ ! $(type -P "gcloud") ]] && { echo "gcloud is NOT in PATH, exiting." 1>&2; exit 1; }

_GCLOUD_SSH_COMMAND="gcloud compute ssh --ssh-flag='-t' --zone=europe-west6-a $GCLOUD_VM_NAME"
_DOCKER_ENTRYPOINT="docker run -v $GCLOUD_VM_DISK:/app/db --entrypoint=node"
_DOCKER_IMAGE="-it gcr.io/hoprassociation/hoprd:$RELEASE_VERSION"
_DOCKER_ARGUMENTS="index.js --data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD' --runAsBootstrap --run 'myAddress native'"

BOOTSTRAP_ADDRESS_COMMAND="$_GCLOUD_SSH_COMMAND -- \"$_DOCKER_ENTRYPOINT $_DOCKER_IMAGE $_DOCKER_ARGUMENTS\""
BOOTSTRAP_ADDRESS=$(eval $BOOTSTRAP_ADDRESS_COMMAND 2>&1 | grep -Eo "0x[a-fA-F0-9]{40}")

echo "BOOTSTRAP_ADDRESS=$BOOTSTRAP_ADDRESS"