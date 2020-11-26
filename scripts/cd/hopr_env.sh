#!/bin/bash
# @param {('*-bootstrap')} GCLOUD_VM_NAME - The name of the container
# @param {('*-bootstrap')} GCLOUD_VM_DISK - The disk of the container
# @param {('**')} RELEASE_VERSION - The version of the docker image
# @param {('**')} BS_PASSWORD - The password of the bootstrap server
# @requires gcloud

set -eu

_GCLOUD_SSH_COMMAND="gcloud compute ssh --ssh-flag='-t' --zone=europe-west6-a $GCLOUD_VM_NAME"
_DOCKER_ENTRYPOINT="docker run -v $GCLOUD_VM_DISK:/app/db --entrypoint=node"
_DOCKER_IMAGE="-it gcr.io/hoprassociation/hoprd:$RELEASE_VERSION"
_DOCKER_ARGUMENTS="index.js --data='/app/db/ethereum/testnet/bootstrap' --password='$BS_PASSWORD' --runAsBootstrap --run 'myAddress hopr'"

BOOTSTRAP_HOPR_ADDRESS_COMMAND="$_GCLOUD_SSH_COMMAND -- \"$_DOCKER_ENTRYPOINT $_DOCKER_IMAGE $_DOCKER_ARGUMENTS\""
BOOTSTRAP_HOPR_ADDRESS=$(eval $BOOTSTRAP_HOPR_ADDRESS_COMMAND)

echo "BOOTSTRAP_HOPR_ADDRESS=$BOOTSTRAP_HOPR_ADDRESS"