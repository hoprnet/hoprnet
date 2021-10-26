#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="build-docker"
source "${mydir}/utils.sh"

declare image_version package_version docker_image environments branch docker_image_full

branch=$(git rev-parse --abbrev-ref HEAD)
image_version="$(date +%s)"
package_version="$("${mydir}/get-package-version.sh")"
docker_image="gcr.io/hoprassociation/hoprd"
docker_image_full="${docker_image}:${image_version}"
environments="$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r ".[] | select(.git_ref==\"refs/heads/${branch}\") | .id")"

if [ -z "${environments}" ]; then
  # return early if no environments are found for branch
  return
fi

cd "${mydir}/../packages/hoprd"

gcloud builds submit --config cloudbuild.yaml \
  --substitutions=_HOPR_PACKAGE_VERSION=${package_version},_HOPR_IMAGE_VERSION=${image_version},_DOCKER_IMAGE=${docker_image}

log "verify bundled version is expected"
declare v=$(docker run --pull always ${docker_image_full} --version 2> /dev/null | sed -n '3p')
[ "${v}" = "${package_version}" ]

log "attach additional tag ${package_version} to docker image ${docker_image_full}"
gcloud container images add-tag ${docker_image_full} ${docker_image}:${package_version}

for environment in ${environments}; do
  log "attach additional tag ${environment} to docker image ${docker_image_full}"
  gcloud container images add-tag ${docker_image_full} ${docker_image}:${environment}
done
