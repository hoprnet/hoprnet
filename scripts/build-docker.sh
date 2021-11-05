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

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-p|--package hoprd|cover-traffic-daemon] [-f|--force]"
  msg
  msg "Use -f to force a Docker build. No additional tags will be applied though."
  msg
}

declare image_version package_version docker_image releases branch docker_image_full package force

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -p|--package)
      package="$2"
      shift 2
      ;;
    -f|--force)
      force="true"
      shift
      ;;
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done

if [ "${package:-}" != "hoprd" ] && [ "${package:-}" != "cover-traffic-daemon" ]; then
  msg "Error: unsupported package"
  usage
  exit 1
fi

branch=$(git rev-parse --abbrev-ref HEAD)
image_version="$(date +%s)"
package_version="$("${mydir}/get-package-version.sh")"
docker_image="gcr.io/hoprassociation/${package}"
docker_image_full="${docker_image}:${image_version}"
releases=""

for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    declare additional_releases="$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .key")"
    releases="${releases} ${additional_releases}"
  fi
done

if [ -z "${releases}" ] && [ "${force:-}" != "true" ]; then
  # return early if no environments are found for branch
  log "no releases were configured for branch ${branch}"
  exit 1
fi

cd "${mydir}/../packages/${package}"

gcloud builds submit --config cloudbuild.yaml \
  --substitutions=_PACKAGE_VERSION=${package_version},_IMAGE_VERSION=${image_version},_DOCKER_IMAGE=${docker_image}

log "verify bundled version"
declare v=$(docker run --pull always ${docker_image_full} --version 2> /dev/null | sed -n '3p')
if [ "${v}" != "${package_version}" ]; then
  log "bundled version ${v}, expected ${package_version}"
  exit 1
fi

if [ -z "${releases}" ] then
  # stopping here after forced build
  log "no releases were configured for branch ${branch}"
  exit 0
fi

log "attach additional tag ${package_version} to docker image ${docker_image_full}"
gcloud container images add-tag ${docker_image_full} ${docker_image}:${package_version}

for release in ${releases}; do
  log "attach additional tag ${release} to docker image ${docker_image_full}"
  gcloud container images add-tag ${docker_image_full} ${docker_image}:${release}
done
