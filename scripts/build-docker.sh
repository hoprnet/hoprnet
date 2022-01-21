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

declare image_version package_version docker_image releases branch package force no_tags

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
releases=""

test_package_version_of_image() {

  local built_docker_image="$1"
  local built_image_version="$2"
  local required_pkg_version="$3"
  local docker_image_full="${built_docker_image}:${built_image_version}"

  log "verify bundled version of ${docker_image_full}"
  local v=$(docker run --pull always -v /var/run/docker.sock:/var/run/docker.sock ${docker_image_full} --version 2> /dev/null | sed -n '3p')
  if [ "${v}" != "${required_pkg_version}" ]; then
    log "bundled version ${v}, expected ${required_pkg_version}"
    return 1
  fi

  log "bundled version in ${docker_image_full} is ${v}"
  return 0
}

build_and_tag_image() {

  local built_docker_image="$1"
  local built_image_version="$2"
  local required_pkg_version="$3"
  local docker_image_full="${built_docker_image}:${built_image_version}"

  log "start building ${docker_image_full}"

  gcloud builds submit --config cloudbuild.yaml \
      --substitutions=_PACKAGE_VERSION=${required_pkg_version},_IMAGE_VERSION=${built_image_version},_DOCKER_IMAGE=${built_docker_image}

  if ! test_package_version_of_image ${built_docker_image} ${built_image_version} ${required_pkg_version} ; then
    exit 0
  fi

  if [ -z "${releases}" ]; then
    # stopping here after forced build
    log "no releases were configured for branch ${branch}"
  else
    if ! [ "${no_tags:-}" = "true" ]; then
      log "attach additional tag ${required_pkg_version} to docker image ${docker_image_full}"
      gcloud container images add-tag ${docker_image_full} ${built_docker_image}:${required_pkg_version}

      for release in ${releases}; do
        log "attach additional tag ${release} to docker image ${docker_image_full}"
        gcloud container images add-tag ${docker_image_full} ${built_docker_image}:${release}
      done
    else
      log "skip tagging as requested"
    fi
  fi

  log "finished build of ${docker_image_full}"
}

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

# go into package directory, make sure to remove prefix and suffix when needed
declare stripped_package=${package#hopr-}
cd "${mydir}/../packages/${stripped_package%-nat}"

# In case we build hoprd-nat, we need to build hoprd first with the same version as a prerequisite (if not already available)
if [[ "${package}" != "hoprd-nat" ]] || ! test_package_version_of_image ${docker_image%-nat} ${image_version} ${package_version} ; then
  build_and_tag_image ${docker_image%-nat} ${image_version} ${package_version}
fi

if [ "${package}" = "hoprd-nat" ]; then
  # Build hoprd-nat with exactly the same version as hoprd
  cd "${mydir}/nat"
  build_and_tag_image ${docker_image} ${image_version} ${package_version}
fi
