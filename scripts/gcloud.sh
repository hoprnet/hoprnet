#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="gcloud"
source "${mydir}/utils.sh"

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

GCLOUD_INCLUDED=1 # So we can test for inclusion
ZONE="--zone=europe-west6-a"
declare gcloud_region="--region=europe-west6"
declare gcloud_disk_name="hoprd-data-disk"

GCLOUD_MACHINE="--machine-type=e2-medium"
GCLOUD_META="--metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true --maintenance-policy=MIGRATE"
GCLOUD_TAGS="--tags=hopr-node,web-client,rest-client,portainer,healthcheck"
GCLOUD_BOOTDISK="--boot-disk-size=20GB --boot-disk-type=pd-standard"
GCLOUD_IMAGE="--image-family=cos-stable --image-project=cos-cloud"

GCLOUD_DEFAULTS="$ZONE $GCLOUD_MACHINE $GCLOUD_META $GCLOUD_TAGS $GCLOUD_BOOTDISK $GCLOUD_IMAGE"

# let keys expire after 1 hour
alias gssh="gcloud compute ssh --force-key-file-overwrite --ssh-key-expire-after=1h --ssh-flag='-t' $ZONE"

# NB: This is useless for getting an IP of a VM
# Get or create an IP address
# $1=VM name
gcloud_get_address() {
  local vm_name="${1}"

  local ip=$(gcloud compute addresses describe ${vm_name} $gcloud_region 2>&1)
  # Google does not return an appropriate exit code :(
  if [ "$(echo "$ip" | grep 'ERROR')" ]; then
    log "No address, creating"
    gcloud compute addresses create ${vm_name} $gcloud_region
    ip=$(gcloud compute addresses describe ${vm_name} $gcloud_region 2>&1)
  fi
  echo $ip | awk '{ print $2 }'
}

# $1=ip
# $2=optional: healthcheck port, defaults to 8080
wait_until_node_is_ready() {
  local ip=${1}
  local port=${2:-8080}
  local cmd="curl --silent --max-time 5 ${ip}:${port}/healthcheck/v1/version"

  # try every 10 seconds for 5 minutes
  log "waiting for VM with IP $1 to have HOPR node up and running"
  try_cmd "${cmd}" 30 10 true
}

# Get external IP for running node or die
# $1 - VM name
gcloud_get_ip() {
  local vm_name="${1}"
  gcloud compute instances list | grep "${vm_name}" | awk '{ print $5 }'
}

# $1=VM name
gcloud_find_vm_with_name() {  
  local vm_name="${1}"
  gcloud compute instances list | grep "${vm_name}" | grep 'RUNNING'
}

# $1 - VM name
# Warning, using `--format='value[](metadata*)` is an unsupported API by gcloud and can change any time.
# More information on https://cloud.google.com/compute/docs/storing-retrieving-metadata
gcloud_get_image_running_on_vm() {
  local vm_name="${1}"

  gcloud compute instances describe ${vm_name} $ZONE \
    --format='value[](metadata.items.gce-container-declaration)' \
    | grep image \
    | tr -s ' ' \
    | cut -f3 -d' '
}

# $1=vm name
# $2=container-image
# $3=disk name
# $4=mount path
# $5=environment_id
gcloud_update_container_with_image() {
  local vm_name="${1}"
  local container_image="${2}"
  local disk_image="${3}"
  local mount_path="${4}"
  local environment_id="${5}"
  local api_token="${HOPRD_API_TOKEN}"
  local password="${BS_PASSWORD}"

  log "${vm_name}"
  log "${container_name}"
  log "${disk_image}"
  log "${mount_path}"

  log "Updating container on vm:${vm_name} - ${$container_name} (disk: ${disk_image}:${mount_path})"
  gcloud compute instances update-container $1 $ZONE \
    --container-image=${container_image} --container-mount-disk name=${disk_image},mount-path="${mount_path}" \
    --container-arg="--admin" \
    --container-arg="--adminHost" --container-arg="0.0.0.0" \
    --container-arg="--announce" \
    --container-arg="--apiToken" --container-arg="${api_token}" \
    --container-arg="--healthCheck" \
    --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
    --container-arg="--identity" --container-arg="${mount_path}/.hopr-identity" \
    --container-arg="--init" \
    --container-arg="--password" --container-arg="${password}" \
    --container-arg="--environment" --container-arg="${environment_id}" \
    --container-arg="--rest" \
    --container-arg="--restHost" --container-arg="0.0.0.0" \
    --container-arg="--run" --container-arg="\"cover-traffic start;daemonize\"" \
    --container-restart-policy=always
}

# $1 - vm name
# $2 - docker image
gcloud_stop() {
  local vm_name="${1}"
  local docker_image="${2}"

  log "Stopping docker image:${docker_image} on vm ${vm_name}"
  gssh ${vm_name} -- "export DOCKER_IMAGE=${docker_image} && docker stop \$(docker ps -q --filter ancestor=\$DOCKER_IMAGE)"
}

# $1 - vm name
# $2 - docker image
gcloud_get_logs() {
  local vm_name="${1}"
  local docker_image="${2}"

  # Docker sucks and gives us warnings in stdout.
  local id=$(gssh ${vm_name} --command "docker ps -q --filter ancestor='${docker_image}' | xargs docker inspect --format='{{.Id}}'" | grep -v 'warning')
  gssh ${vm_name} --command "docker logs $id"
}

# $1 - vm name
gcloud_cleanup_docker_images() {
  local vm_name="${1}"

  log "pruning docker images on host ${vm_name}"
  gssh "${vm_name}" --command "sudo docker system prune -a -f"
}

# $1 - template name
# $2 - container image
# $3 - optional: environment id
# $4 - optional: api token
# $5 - optional: password
# $6 - optional: announce
# $7 - optional: private key
# $8 - optional: no args
gcloud_create_or_update_instance_template() {
  local args name mount_path image rpc api_token password host_path no_args private_key announce
  local extra_args=""

  name="${1}"
  image="${2}"
  environment_id="${3:-}"

  # these parameters are only used by hoprd nodes
  api_token="${4:-}"
  password="${5:-}"

  # if set, let the node announce with a routable address on-chain
  announce="${6:-}"

  # this parameter is mostly used on by CT nodes, although hoprd nodes also
  # support it
  private_key="${7:-}"

  # if set no additional arguments are used to start the container
  no_args="${8:-}"

  args=""
  # the environment is optional, since each docker image has a default environment set
  if [ -n "${environment_id}" ]; then
    args="--container-arg=\"--environment\" --container-arg=\"${environment_id}\""
  fi

  if [ -n "${api_token}" ]; then
    extra_args="${extra_args} --container-arg=\"--apiToken\" --container-arg=\"${api_token}\""
  fi

  if [ -n "${password}" ]; then
    extra_args="${extra_args} --container-arg=\"--password\" --container-arg=\"${password}\""
  fi

  if [ -n "${private_key}" ]; then
    extra_args="${extra_args} --container-arg=\"--privateKey\" --container-arg=\"${private_key}\""
  fi

  if [ -n "${announce}" ]; then
    extra_args="${extra_args} --container-arg=\"--announce\""
  fi

  mount_path="/app/db"
  host_path="/var/hoprd"

  log "checking for instance template ${name}"
  if gcloud compute instance-templates describe "${name}" --quiet >/dev/null; then
    log "instance template ${name} already present"
    gcloud_delete_instance_template "${name}"
  fi

  log "creating instance template ${name}"

  if [ "${no_args}" = "true" ]; then
    eval gcloud compute instance-templates create-with-container "${name}" \
      --machine-type=e2-medium \
      --metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true \
      --maintenance-policy=MIGRATE \
      --tags=hopr-node,web-client,rest-client,portainer,healthcheck \
      --boot-disk-size=20GB \
      --boot-disk-type=pd-balanced \
      --image-family=cos-stable \
      --image-project=cos-cloud \
      --container-image="${image}" \
      --container-env=^,@^DEBUG=hopr\*,-hopr-connect\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-mount-host-path=mount-path="${mount_path}",host-path="${host_path}" \
      --container-mount-host-path=mount-path=/var/run/docker.sock,host-path=/var/run/docker.sock \
      --container-restart-policy=always \
      ${args} \
      ${extra_args}
  else
    eval gcloud compute instance-templates create-with-container "${name}" \
      --machine-type=e2-medium \
      --metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true \
      --maintenance-policy=MIGRATE \
      --tags=hopr-node,web-client,rest-client,portainer,healthcheck \
      --boot-disk-size=20GB \
      --boot-disk-type=pd-balanced \
      --image-family=cos-stable \
      --image-project=cos-cloud \
      --container-image="${image}" \
      --container-env=^,@^DEBUG=hopr\*,@NODE_OPTIONS=--max-old-space-size=4096,@GCLOUD=1 \
      --container-mount-host-path=mount-path="${mount_path}",host-path="${host_path}" \
      --container-mount-host-path=mount-path=/var/run/docker.sock,host-path=/var/run/docker.sock \
      --container-restart-policy=always \
      --container-arg="--admin" \
      --container-arg="--adminHost" --container-arg="0.0.0.0" \
      --container-arg="--healthCheck" \
      --container-arg="--healthCheckHost" --container-arg="0.0.0.0" \
      --container-arg="--identity" --container-arg="${mount_path}/.hopr-identity" \
      --container-arg="--init" \
      --container-arg="--rest" \
      --container-arg="--restHost" --container-arg="0.0.0.0" \
      ${args} \
      ${extra_args}
  fi
}

# $1 - template name
gcloud_delete_instance_template() {
  local name="${1}"

  log "deleting instance template ${name}"
  gcloud compute instance-templates delete "${name}" --quiet
}

# $1=group name
# $2=group size
# $3=template name
gcloud_create_or_update_managed_instance_group() {
  local name="${1}"
  local size="${2}"
  local template="${3}"

  log "checking for managed instance group ${name}"
  if gcloud compute instance-groups managed describe "${name}" ${gcloud_region} --quiet; then
    log "managed instance group ${name} already present, updating..."
    gcloud compute instance-groups managed rolling-action start-update \
      "${name}"\
      --version=template=${template} \
      ${gcloud_region}
  else
    log "creating managed instance group ${name}"
    gcloud compute instance-groups managed create "${name}" \
      --base-instance-name "${name}-vm" \
      --size ${size} \
      --template "${template}" \
      --instance-redistribution-type=NONE \
      ${gcloud_region}
  fi

  log "waiting for managed instance group ${name}"
  gcloud compute instance-groups managed wait-until "${name}" \
    --stable \
    ${gcloud_region}
}

# $1=group name
gcloud_delete_managed_instance_group() {
  local name="${1}"

  log "deleting managed instance group ${name}"
  gcloud compute instance-groups managed delete "${name}" \
    --quiet \
    ${gcloud_region}
}

# $1=group name
gcloud_get_managed_instance_group_instances_ips() {
  local name="${1}"

  gcloud compute instance-groups list-instances "${name}" \
    ${gcloud_region} --uri | \
    xargs -P `nproc` -I '{}' gcloud compute instances describe '{}' \
      --flatten 'networkInterfaces[].accessConfigs[]' \
      --format 'csv[no-heading](networkInterfaces.accessConfigs.natIP)'
}
