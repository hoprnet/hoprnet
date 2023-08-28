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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.

GCLOUD_INCLUDED=1 # So we can test for inclusion
# using Belgium for better access to more VM types
ZONE="--zone=europe-west4-c"
declare gcloud_region="--region=europe-west4"
declare gcloud_disk_name="hoprd-data-disk"

# use CPU optimized machine type
GCLOUD_MACHINE="--machine-type=n2-standard-2"
GCLOUD_META="--metadata=google-logging-enabled=true,google-monitoring-enabled=true,enable-oslogin=true --maintenance-policy=MIGRATE"
GCLOUD_TAGS="--tags=hopr-node,web-client,rest-client,portainer,healthcheck"
GCLOUD_BOOTDISK="--boot-disk-size=20GB --boot-disk-type=pd-standard"
GCLOUD_IMAGE="--image-family=cos-stable --image-project=cos-cloud"

GCLOUD_DEFAULTS="$ZONE $GCLOUD_MACHINE $GCLOUD_META $GCLOUD_TAGS $GCLOUD_BOOTDISK $GCLOUD_IMAGE"

# let keys expire after 1 hour
declare gssh="gcloud compute ssh --force-key-file-overwrite --ssh-key-expire-after=1h --ssh-flag=-t"

# $1=ip
# $2=optional: healthcheck port, defaults to 8080
wait_until_node_is_ready() {
  local ip="${1}"
  local port="${2:-8080}"
  local cmd="curl --silent --max-time 5 ${ip}:${port}/healthcheck/v1/version"
  local vsn

  # try every 10 seconds for 5 minutes
  log "waiting for VM with IP ${ip} to have HOPRd node up and running"
  vsn="$(try_cmd "${cmd}" 30 10)"
  log "VM with IP ${ip} is running HOPRd v${vsn}"
}

# Get external IP for running node or die
# $1 - VM instance uri
gcloud_get_ip() {
  local instance_uri="${1}"

  gcloud compute instances describe "${instance_uri}" \
    --flatten 'networkInterfaces[].accessConfigs[]' \
    --format 'csv[no-heading](networkInterfaces.accessConfigs.natIP)'
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

  gcloud compute instances describe "${vm_name}" $ZONE \
    --format='value[](metadata.items.gce-container-declaration)' \
    | grep image \
    | tr -s ' ' \
    | cut -f3 -d' '
}


# $1 - vm name
# $2 - docker image
gcloud_stop() {
  local vm_name="${1}"
  local docker_image="${2}"

  log "Stopping docker image:${docker_image} on vm ${vm_name}"
  ${gssh} "${vm_name}" -- "export DOCKER_IMAGE=${docker_image} && docker stop \$(docker ps -q --filter ancestor=\$DOCKER_IMAGE)"
}

# $1 - vm name
# $2 - docker image
gcloud_get_logs() {
  local vm_name="${1}"
  local docker_image="${2}"

  # Docker sucks and gives us warnings in stdout.
  local id=$(${gssh} "${vm_name}" --command "docker ps -q --filter ancestor='${docker_image}' | xargs docker inspect --format='{{.Id}}'" | grep -v 'warning')
  ${gssh} "${vm_name}" --command "docker logs $id"
}

# $1 - vm name
gcloud_cleanup_docker_images() {
  local vm_name="${1}"

  log "pruning docker images on host ${vm_name}"
  ${gssh} "${vm_name}" --command "sudo docker system prune -a -f"
}

# $1 - template name
gcloud_create_instance_template() {
  local name="${1}"

  log "checking for instance template ${name}"
  if gcloud compute instance-templates describe "${name}" --quiet 2> /dev/null; then
    log "instance template ${name} already present"
    return 0
  fi

  log "creating instance template ${name}"
  eval gcloud compute instance-templates create "${name}" \
      ${GCLOUD_MACHINE} \
      --maintenance-policy=MIGRATE \
      --tags=hopr-node,web-client,rest-client,portainer,healthcheck \
      --boot-disk-device-name=boot-disk \
      --boot-disk-size=20GB \
      --boot-disk-type=pd-balanced \
      --image-family=debian-11 \
      --image-project=hoprassociation \
      --maintenance-policy=MIGRATE
}

# $1 - template name
gcloud_delete_instance_template() {
  local name="${1}"

  log "deleting instance template ${name}"
  gcloud compute instance-templates delete "${name}" --quiet
}

# $1=group name
# $2=group size
gcloud_create_or_update_managed_instance_group() {
  local name="${1}"
  local size="${2}"

  log "checking for managed instance group ${name}"
  if gcloud compute instance-groups managed describe "${name}" ${gcloud_region} --quiet 2> /dev/null; then
    # get current instance template name
    local first_instance_name="$(gcloud compute instance-groups list-instances \
      "${name}" ${gcloud_region} --format=json | jq -r '.[0].instance')"
    local previous_template="$(gcloud compute instances describe \
      "${first_instance_name}" --format=json | \
      jq '.metadata.items[] | select(.key=="instance-template") | .value' | \
      tr -d '"' | awk -F'/' '{ print $5; }')"

    log "managed instance group ${name} already present, updating..."

    # ensure instances are not replaced to prevent IP re-assignments
    gcloud beta compute instance-groups managed rolling-action start-update \
      "${name}"\
      --version=template="${name}" \
      --minimal-action=restart \
      --most-disruptive-allowed-action=restart \
      --replacement-method=recreate \
      ${gcloud_region}
  else
    log "creating managed instance group ${name}"
    gcloud compute instance-groups managed create "${name}" \
      --base-instance-name "${name}-vm" \
      --size "${size}" \
      --template "${name}" \
      --instance-redistribution-type=NONE \
      --stateful-disk "device-name=boot-disk,auto-delete=on-permanent-instance-deletion" \
      ${gcloud_region}
  fi

  log "waiting for managed instance group ${name}"
  gcloud compute instance-groups managed wait-until "${name}" \
    --stable \
    ${gcloud_region}

  log "reserve all external addresses of the instance group ${name} instances"
  for instance_uri in $(gcloud compute instance-groups list-instances "${name}" ${gcloud_region} --uri); do
    local instance_name=$(gcloud compute instances describe "${instance_uri}" --format 'csv[no-heading](name)')
    local instance_ip=$(gcloud_get_ip "${instance_uri}")

    gcloud_reserve_static_ip_address "${instance_name}" "${instance_ip}"
  done
}

# $1=group name
gcloud_delete_managed_instance_group() {
  local name="${1}"

  log "un-reserve all external addresses of the instance group ${name} instances"
  for instance_uri in $(gcloud compute instance-groups list-instances "${name}" ${gcloud_region} --uri); do
    local instance_name=$(gcloud compute instances describe "${instance_uri}" --format 'csv[no-heading](name)')
    local instance_ip=$(gcloud_get_ip "${instance_uri}")

    gcloud_delete_static_ip_address "${instance_name}"
  done

  log "deleting managed instance group ${name}"
  gcloud compute instance-groups managed delete "${name}" \
    --quiet \
    ${gcloud_region}
}

# $1=group name
gcloud_get_managed_instance_group_instances_ips() {
  local name="${1}"
  local nproc_cmd

  if command -v nproc 1> /dev/null ; then
    nproc_cmd="nproc"
  elif command -v sysctl 1> /dev/null ; then
    nproc_cmd="sysctl -n hw.logicalcpu"
  else
    # Default to single core
    nproc_cmd="echo 1"
  fi

  export -f gcloud_get_ip
  gcloud compute instance-groups list-instances "${name}" \
    ${gcloud_region} --sort-by=instance --uri | \
    xargs -P $(${nproc_cmd}) -I '{}' bash -c "gcloud_get_ip '{}'"
}

# returns a JSON list of strings
# $1=group name
gcloud_get_managed_instance_group_instances_names() {
  local name="${1}"

  gcloud compute instance-groups list-instances "${name}" ${gcloud_region} --sort-by=instance \
    --format=json | jq 'map(.instance)'
}

# returns a JSON list of key/value paris
# $1=instance name
gcloud_get_instance_metadata() {
  local name="${1}"

  gcloud compute instances describe "${name}" --format=json | jq ".metadata.items"
}

# $1=instance name
# $2=comma separated list of metadata to add
gcloud_add_instance_metadata() {
  local name="${1}"
  local metadata="${2}"

  gcloud compute instances add-metadata "${name}" --metadata="${metadata}"
}

# $1 = instance name
gcloud_get_node_info_metadata() {
  local instance_name="${1}"

  # filter by prefix hopr- and return results as object
  gcloud_get_instance_metadata "${instance_name}" | jq 'map(select(.key | startswith("hopr-"))) | from_entries'
}

# $1=instance name
# $2=comma separated list of metadata keys to remove
gcloud_remove_instance_metadata() {
  local name="${1}"
  local keys="${2}"

  gcloud compute instances remove-metadata "${name}" --keys="${keys}"
}

gcloud_get_unused_static_ip_addresses() {
  gcloud compute addresses list --filter='status != IN_USE' --format=json ${gcloud_region}
}

# $1=address name
gcloud_delete_static_ip_address() {
  local address="${1}"

  gcloud compute addresses delete "${address}" --quiet ${gcloud_region}
}

# $1=address name
# $2=ip
gcloud_reserve_static_ip_address() {
  local address="${1}"
  local ip="${2}"

  if gcloud compute addresses describe ${gcloud_region} "${address}" 2> /dev/null; then
    # already reserved, no-op
    :
  else
    gcloud compute addresses create "${address}" --addresses="${ip}" ${gcloud_region}
  fi
}

# $1=instance name
# $2=command to execute
gcloud_execute_command_instance() {
  local name="${1}"
  local command="${2}"

  ${gssh} "${name}" --command "${command}"
}

# $1=flag name
gcloud_isset_project_flag() {
  gcloud compute project-info describe --format=json | jq "any(.commonInstanceMetadata.items[]; .key ==\"${1}\")"
}

# $1=flag name
# $2=flag value
gcloud_set_project_flag() {
  gcloud compute project-info add-metadata --metadata="${1}=${2}" 2> /dev/null
}

# $1=flag name
gcloud_unset_project_flag() {
  gcloud compute project-info remove-metadata --keys="${1}" 2> /dev/null
}