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
  msg "Usage: $0 [-h|--help] [-p|--package hoprd|hoprd-nat|hopr-cover-traffic-daemon] [-f|--force] [-n|--no-tags]"
  msg
  msg "Use -f to force a Docker build even though no environment can be found. This is useful for local testing. No additional docker tags will be applied though if no environment has been found which is in contrast to the normal execution of the script."
  msg
}

declare image_version package_version docker_image releases branch docker_image_full package force no_tags

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
    -n|--no-tags)
      no_tags="true"
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

if [ "${package:-}" != "hoprd" ] && [ "${package:-}" != "hoprd-nat" ] && [ "${package:-}" != "hopr-cover-traffic-daemon" ]; then
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

if [ "${package}" = "hoprd-nat" ]; then
  # First build the hoprd image we depend on
  cd "${mydir}/../packages/hoprd"
  gcloud builds submit --config cloudbuild.yaml \
    --substitutions=_PACKAGE_VERSION=${package_version},_IMAGE_VERSION=${image_version},_DOCKER_IMAGE="gcr.io/hoprassociation/hoprd"

  cd "${mydir}/nat"
else
  # go into package directory, make sure to remove prefix when needed
  cd "${mydir}/../packages/${package#hopr-}"
fi

gcloud builds submit --config cloudbuild.yaml \
  --substitutions=_PACKAGE_VERSION=${package_version},_IMAGE_VERSION=${image_version},_DOCKER_IMAGE=${docker_image}

log "verify bundled version of ${docker_image_full}"
declare v=$(docker run --pull always -v /var/run/docker.sock:/var/run/docker.sock ${docker_image_full} --version 2> /dev/null | sed -n '3p')
if [ "${v}" != "${package_version}" ]; then
  log "bundled version ${v}, expected ${package_version}"
  exit 1
fi

if [ -z "${releases}" ]; then
  # stopping here after forced build
  log "no releases were configured for branch ${branch}"
  exit 0
fi

if ! [ "${no_tags:-}" = "true" ]; then
  log "attach additional tag ${package_version} to docker image ${docker_image_full}"
  gcloud container images add-tag ${docker_image_full} ${docker_image}:${package_version}

  for release in ${releases}; do
    log "attach additional tag ${release} to docker image ${docker_image_full}"
    gcloud container images add-tag ${docker_image_full} ${docker_image}:${release}
  done
else
  log "skip tagging as requested"
fi

