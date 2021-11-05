#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="deploy"
source "${mydir}/utils.sh"
source "${mydir}/testnet.sh"

declare branch cluster_size package_version docker_image

branch=$(git rev-parse --abbrev-ref HEAD)
cluster_size=3
package_version=$(${mydir}/get-package-version.sh)
docker_image="gcr.io/hoprassociation/hoprd"

# ---- On Deployment -----
#
# This finds matching entries in packages/hoprd/releases.json and deploys accordingly
#
# ENV Variables:
# - FUNDING_PRIV_KEY: funding private key, raw
# - BS_PASSWORD: database password

_jq() {
  echo "$1" | base64 --decode | jq -r "$2"
}

echo "Looking for releases to deploy from branch ${branch}"

# iterate through releases and find entries which match to the current branch

for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    for row in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | @base64"); do
      declare release_id deprecated environment_id version_major version_minor docker_image_full token_contract_address

      release_id=$(_jq "${row}" ".key")
      deprecated=$(_jq "${row}" ".value.deprecated")
      environment_id=$(_jq "${row}" ".value.environment_id")
      version_major=$(_jq "${row}" ".value.version_major")
      version_minor=$(_jq "${row}" ".value.version_minor")
      docker_image_full="${docker_image}:${release_id}"
      token_contract_address=$(cat "${mydir}/../packages/core/protocol-config.json" | jq -r ".environments.\"${environment_id}\".token_contract_address")

      if [ "${deprecated}" == "true" ]; then
        log "${release_id} deprecated, skipping deployment"
        continue
      fi

      declare version_maj_min cluster_name
      cluster_name="${release_id}"
      if [ "${version_major}" != "null" ] && [ "${version_minor}" != "null" ]; then
        version_maj_min="${version_major}.${version_minor}"
        cluster_name="${cluster_name}-${version_maj_min//./-}"
      else
        version_maj_min=""
      fi

      cluster_template_name="${cluster_name}-${package_version//./-}"

      log "deploying release ${release_id}"
      log "\tversion: ${version_maj_min}"
      log "\tenvironment ${environment_id}"
      log "\tdocker image: ${docker_image_full}"
      log "\tcluster name: ${cluster_name}"
      log "\tcluster template name: ${cluster_template_name}"

      gcloud_create_or_update_instance_template "${cluster_template_name}" \
        "${docker_image_full}" \
        "${environment}" \
        "${api_token}" \
        "${password}"

      gcloud_create_or_update_managed_instance_group "${cluster_name}" \
        ${cluster_size} \
        "${cluster_template_name}"

      # get IPs of VMs which run hoprd
      declare node_ips
      node_ips=$(gcloud_get_managed_instance_group_instances_ips "${cluster_name}")
      declare node_ips_arr=( ${node_ips} )

      # fund nodes
      declare eth_address
      for ip in ${node_ips}; do
        wait_until_node_is_ready "${ip}"
        eth_address=$(get_eth_address "${ip}")
        fund_if_empty "${eth_address}" "${environment_id}" "${token_contract_address}"
      done
    done
  fi
done
