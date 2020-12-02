#!/bin/bash
# @param {('**')}               GCLOUD_VM_NAME            - The name of the virtual machine to modify/create
# @param {('*.*.*')}            RELEASE_VERSION           - The name of the virtual machine to modify/create
# @requires gcloud
set -eu
[[ ! $(type -P "gcloud") ]] && { echo "gcloud is NOT in PATH, exiting." 1>&2; exit 1; }

if [[ -z "${GCLOUD_VM_NAME}" ]]; then
  echo 'No action has been provide as GCLOUD_VM_NAME env, exit.'
  exit 0
fi

_GCLOUD_SSH_COMMAND="gcloud compute ssh --zone=europe-west6-a $GCLOUD_VM_NAME"
_DOCKER_ENTRYPOINT='docker stop'
_DOCKER_ARGUMENTS=$(echo "\$(docker ps -q --filter \"ancestor=gcr.io/hoprassociation/hoprd:$RELEASE_VERSION\")")

DOCKER_STOP_COMMAND="$_GCLOUD_SSH_COMMAND -- '$_DOCKER_ENTRYPOINT $_DOCKER_ARGUMENTS'"

eval $DOCKER_STOP_COMMAND 2>&1
