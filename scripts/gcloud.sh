#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

GCLOUD_INCLUDED=1 # So we can test for inclusion
ZONE="--zone=europe-west6-a"
REGION="--region=europe-west6"

GCLOUD_MACHINE="--machine-type=e2-medium"
GCLOUD_META="--metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true --maintenance-policy=MIGRATE"
GCLOUD_TAGS="--tags=hopr-node,web-client,rest-client,portainer,healthcheck"
GCLOUD_BOOTDISK="--boot-disk-size=20GB --boot-disk-type=pd-standard"
GCLOUD_IMAGE="--image-family=cos-stable --image-project=cos-cloud"

GCLOUD_DEFAULTS="$ZONE $GCLOUD_MACHINE $GCLOUD_META $GCLOUD_TAGS $GCLOUD_BOOTDISK $GCLOUD_IMAGE"

alias gssh="gcloud compute ssh --ssh-flag='-t' $ZONE"

# NB: This is useless for getting an IP of a VM
# Get or create an IP address
# $1 = name
gcloud_get_address() {
  local ip=$(gcloud compute addresses describe $1 $REGION 2>&1)
  # Google does not return an appropriate exit code :(
  if [ "$(echo "$ip" | grep 'ERROR')" ]; then
    echo "No address, creating" 1>&2
    gcloud compute addresses create $1 $REGION
    local ip=$(gcloud compute addresses describe $1 $REGION 2>&1)
  fi
  echo $ip | awk '{ print $2 }'
}

# Waits until the node is ready (port 8080 healthcheck by default)
# $1 = ip
wait_until_node_is_ready() {
  is_ready=0
  while [ $is_ready -eq 0 ]; do
    if curl $1:8080/healthcheck/v1/version; then
      echo "VM with IP $1 has a HOPR node up and running"
      is_ready=1
    else
      echo "VM with IP $1 is not ready, sleeping for 10s"
      sleep 10
    fi
  done
}

# Get external IP for running node or die
# $1 - name
gcloud_get_ip() {
  gcloud compute instances list | grep "$1" | awk '{ print $5 }'
}

# $1 = VM name
gcloud_find_vm_with_name() {
  gcloud compute instances list | grep "$1" | grep 'RUNNING'
}

# $1 - VM name
# Warning, using `--format='value[](metadata*)` is an unsupported API by gcloud and can change any time.
# More information on https://cloud.google.com/compute/docs/storing-retrieving-metadata
gcloud_get_image_running_on_vm() {
  gcloud compute instances describe $1 $ZONE \
    --format='value[](metadata.items.gce-container-declaration)' \
    | grep image \
    | tr -s ' ' \
    | cut -f3 -d' '
}

# $1 = vm name
# $2 = container-image
# $3 = disk name
# $4 = mount path
# $5 = chain provider
gcloud_update_container_with_image() {
  local rpc=${5}
  local api_token="${HOPRD_API_TOKEN}"
  local password="${BS_PASSWORD}"

  echo "Updating container on vm:$1 - $2 (disk: $3:$4)"
  gcloud compute instances update-container $1 $ZONE \
    --container-image=$2 --container-mount-disk name=$3,mount-path="$4" \
    --container-arg="--admin" \
    --container-arg="--adminHost" --container-arg="0.0.0.0" \
    --container-arg="--announce" \
    --container-arg="--apiToken" --container-arg="${api_token}" \
    --container-arg="--healthCheck" \
    --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
    --container-arg="--identity" --container-arg="${4}/.hopr-identity" \
    --container-arg="--init" \
    --container-arg="--password" --container-arg="${password}" \
    --container-arg="--provider" --container-arg="${rpc}" \
    --container-arg="--rest" \
    --container-arg="--restHost" --container-arg="0.0.0.0" \
    --container-arg="--run" --container-arg="\"cover-traffic start;daemonize\"" \
    --container-restart-policy=always
  sleep 30s
}

# $1 - vm name
# $2 - docker image
gcloud_stop() {
  echo "Stopping docker image:$2 on vm $1"
  gssh $1 -- "export DOCKER_IMAGE=$2 && docker stop \$(docker ps -q --filter ancestor=\$DOCKER_IMAGE)"
}

# $1 - vm name
# $2 - docker image
gcloud_get_logs() {
  # Docker sucks and gives us warnings in stdout.
  local id=$(gcloud compute ssh $ZONE $1 --command "docker ps -q --filter ancestor='$2' | xargs docker inspect --format='{{.Id}}'" | grep -v 'warning')
  gcloud compute ssh $ZONE $1 --command "docker logs $id"
}

# $1 - vm name
gcloud_cleanup_docker_images() {
  gcloud compute ssh $ZONE "$1" --command "sudo docker system prune -a -f"
}
