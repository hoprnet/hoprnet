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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"
# shellcheck disable=SC1090
source "${mydir}/testnet.sh"

declare branch cluster_size docker_image

: "${HOPRD_API_TOKEN:?"env var missing"}"
: "${HOPRD_PASSWORD:?"env var missing"}"
: "${FAUCET_SECRET_API_KEY:?"env var missing"}"

# docker_image and cluster_size are configurable through script arguments
docker_image="${1:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:latest}"
cluster_size=${2:-3}
branch=${3:-master} # bratislava , providence
cluster_tag=${4:-} # optional cluster tag


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
      declare release_id deprecated network version_major version_minor

      release_id=$(_jq "${row}" ".key")
      deprecated=$(_jq "${row}" ".value.deprecated")
      network=$(_jq "${row}" ".value.network")
      version_major=$(_jq "${row}" ".value.version_major")
      version_minor=$(_jq "${row}" ".value.version_minor")


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

      cluster_template_name="${cluster_name}"

      log "deploying release ${release_id}"
      log "\tversion: ${version_maj_min}"
      log "\tnetwork id: ${network}"
      log "\tdocker image: ${docker_image}"
      log "\tcluster name: ${cluster_name}"
      log "\tcluster template name: ${cluster_template_name}"

      # check if the tag ends with -nat
      if [[ "${cluster_tag}" =~ "-nat$" ]]; then
        log "\tNATed node, no announcements"
        "${mydir}"/setup-gcloud-cluster.sh \
          "${network}" \
          "" \
          "${cluster_name}" \
          "${docker_image}" \
          "${cluster_size}" \
          "${cluster_template_name}" \
          ""
      else
        # announce on-chain with routable address
        "${mydir}"/setup-gcloud-cluster.sh \
          "${network}" \
          "" \
          "${cluster_name}" \
          "${docker_image}" \
          "${cluster_size}" \
          "${cluster_template_name}" \
          "true"
      fi
    done
  fi
done
