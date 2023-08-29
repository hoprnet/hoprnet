#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC1090
source "${mydir}/gcloud.sh"

declare cluster_template_name=${1?"Missing parameter cluster_template"}
declare identities_path="${2?"missing parameter <identities_path>"}"

identities=($(ls ${identities_path}))
instances=($(gcloud compute instance-groups list-instances ${cluster_template_name} ${gcloud_region} --format=json | jq 'map(.instance)' | sed 's/.*zones\//"/' | sed 's/\/instances\//;/' | jq -r '@csv' | tr -d '\"' | sed 's/,/ /g'))

setup_node(){
  zone=${1}
  instance=${2}
  identity=${3}
  echo "Setting up ${instance} with $identities_path/$identity"
  gcloud compute ssh --zone ${zone} ${instance} --command 'sudo rm -f /opt/hoprd/{.hoprd.id,.env}'
  gcloud compute scp --zone ${zone}  $identities_path/$identity/{.hoprd.id,.env} ${instance}:/opt/hoprd/
  echo "Ensure host ip is set in environment file"
  gcloud compute ssh --zone ${zone} ${instance} --command '
    ip=$(curl -s -H "Metadata-Flavor:Google" http://metadata/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip)
    sudo echo "HOPRD_HOST=${ip}:9091" >> /opt/hoprd/.env
  '
  gcloud compute ssh --zone ${zone} ${instance} --command 'sudo chown root:root -R /opt/hoprd'
  echo "Restarting VM ${instance}"
  gcloud compute ssh --zone ${zone} ${instance} --command 'sudo service hoprd restart'
}

# Iterate through all VM instances
for index in "${!instances[@]}"; do
  zone=$(echo "${instances[$index]}" | cut -d ";" -f 1)
  instance=$(echo "${instances[$index]}" | cut -d ";" -f 2)
  identity="${identities[$index]}"
  setup_node ${zone} ${instance} ${identity} &
done

wait
