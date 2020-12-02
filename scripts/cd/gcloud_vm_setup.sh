#!/bin/bash
# @param {('update'|'create')}  GCLOUD_ACTION_CONTAINER   - The action to execute on the vm
# @param {('**')}               GCLOUD_VM_NAME            - The name of the virtual machine to modify/create
# @param {('**')}               BS_PASSWORD               - The password of the bootstrap server
# @param {('*.*.*')}            RELEASE_VERSION           - The release to use on the machine
# @param {('master'|'**')}      RELEASE_NAME              - The cleaned up version of release
# @requires gcloud
set -e
[[ ! $(type -P "gcloud") ]] && { echo "gcloud is NOT in PATH, exiting." 1>&2; exit 1; }

if [[ -z "${GCLOUD_ACTION_CONTAINER}" ]]; then
  echo 'No action has been provide as GCLOUD_ACTION_CONTAINER env, exit.'
  exit 0
else
  if [ "$GCLOUD_ACTION_CONTAINER" == 'update' ]; then
    gcloud compute instances update-container $GCLOUD_VM_NAME \
      --zone=europe-west6-a \
      --container-image=gcr.io/hoprassociation/hoprd:$RELEASE_VERSION \
      --container-mount-disk name=bs-$RELEASE_NAME,mount-path="/app/db"
  elif [ "$GCLOUD_ACTION_CONTAINER" == 'create' ]; then
    gcloud compute instances create-with-container ${{ env.GCLOUD_VM_NAME }} \
      --zone=europe-west6-a \
      --machine-type=e2-medium \
      --network-interface=address=$RELEASE_IP,network-tier=PREMIUM,subnet=default \
      --metadata=google-logging-enabled=true --maintenance-policy=MIGRATE \
      --create-disk name=bs-$RELEASE_NAME,size=10GB,type=pd-ssd,mode=rw \
      --container-mount-disk mount-path="/app/db" \
      --tags=hopr-node,web-client,portainer \
      --boot-disk-size=10GB --boot-disk-type=pd-standard \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096 \
      --container-image=gcr.io/hoprassociation/hoprd:$RELEASE_VERSION \
      --container-arg="--password" --container-arg="$BS_PASSWORD" \
      --container-arg="--env" --container-arg="matic" \
      --container-arg="--init" --container-arg="true" \
      --container-arg="--runAsBootstrap" --container-arg="true" \
      --container-arg="--admin" \
      --container-restart-policy=always
  else
    echo 'Unknown action described by GCLOUD_ACTION_CONTAINER, exit'
    exit 0
  fi
fi