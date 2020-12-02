#!/bin/bash
# @param {('update'|'create')}  GCLOUD_ACTION_CONTAINER   - The action to execute on the vm
# @param {('**')}               GCLOUD_VM_NAME            - The name of the virtual machine to modify/create
# @param {('*.*.*')}            RELEASE_VERSION           - The release to use on the machine
# @param {('master'|'**')}      RELEASE_NAME              - The cleaned up version of release
# @requires gcloud
set -e

if [[ -z "${GCLOUD_ACTION_CONTAINER}" ]]; then
  echo 'No action has been provide as GCLOUD_ACTION_CONTAINER env, exit.'
  exit 0
else
  if [ "$GCLOUD_ACTION_CONTAINER" == 'update' ]; then
    echo 'Updating container...'
    gcloud compute instances update-container $GCLOUD_VM_NAME \
      --zone=europe-west6-a \
      --container-image=gcr.io/hoprassociation/hoprd:$RELEASE_VERSION \
      --container-mount-disk name=bs-$RELEASE_NAME,mount-path="/app/db"
    echo 'Finished updating container'
  elif [ "$GCLOUD_ACTION_CONTAINER" == 'create' ]; then
    echo 'Creating container...'
  else
    echo 'Unknown action described by GCLOUD_ACTION_CONTAINER, exit'
    exit 0
  fi
fi