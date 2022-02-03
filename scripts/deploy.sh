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

: ${HOPRD_API_TOKEN:?"env var missing"}
: ${HOPRD_PASSWORD:?"env var missing"}
: ${FUNDING_PRIV_KEY:?"env var missing"}

# docker_image and cluster_size are configurable through script arguments
docker_image="${1:-gcr.io/hoprassociation/hoprd}"
cluster_size=${2:-3}
cluster_tag=${3:-} # optional cluster tag

branch=$(git rev-parse --abbrev-ref HEAD)
package_version=$(${mydir}/get-package-version.sh)
api_token="${HOPRD_API_TOKEN}"
password="${HOPRD_PASSWORD}"

_jq() {
  echo "$1" | base64 --decode | jq -r "$2"
}

echo "Looking for releases to deploy from branch ${branch}"

# iterate through releases and find entries which match to the current branch

for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    for row in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | @base64"); do
      declare release_id deprecated environment_id version_major version_minor docker_image_full

      release_id=$(_jq "${row}" ".key")
      deprecated=$(_jq "${row}" ".value.deprecated")
      environment_id=$(_jq "${row}" ".value.environment_id")
      version_major=$(_jq "${row}" ".value.version_major")
      version_minor=$(_jq "${row}" ".value.version_minor")
      docker_image_full="${docker_image}:${release_id}"

      if [ "${deprecated}" == "true" ]; then
        log "${release_id} deprecated, skipping deployment"
        continue
      fi

      declare version_maj_min cluster_name
      cluster_name="${release_id}${cluster_tag}"
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

      if [ "${cluster_tag}" = "-nat" ]; then
        log "\tNATed node, no announcements"
        ${mydir}/setup-gcloud-cluster.sh \
          "${environment_id}" \
          "" \
          "${cluster_name}" \
          "${docker_image_full}" \
          "2" \
          "${cluster_template_name}"
      else
        # announce on-chain with routable address
        ${mydir}/setup-gcloud-cluster.sh \
          "${environment_id}" \
          "" \
          "${cluster_name}" \
          "${docker_image_full}" \
          "3" \
          "${cluster_template_name}" \
          "true"
      fi
    done
  fi
done
