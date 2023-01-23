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
  msg "Usage: $0 [-h|--help] [-f|--force] [-n|--no-tags] [-l|--local] [-i|--image]"
  msg
  msg "This script builds all docker images defined within the monorepo using Google Cloud Build."
  msg "The images are defined in ${mydir}/../cloudbuild.yaml"
  msg
  msg "Use -l to build the images locally instead and not publish them to a
  remote Docker repository. In addition -i can be used to build a single image locally instead of all images."
  msg "Supported values for -p are 'hoprd', 'hoprd-nat', 'anvil', 'pluto', 'pluto-complete'"
  msg
  msg "Use -f to force a Docker builds even though no environment can be found. This is useful for local testing. No additional docker tags will be applied though if no environment has been found which is in contrast to the normal execution of the script."
  msg
}

declare image_version package_version releases branch force no_tags local_build
declare image_name=""

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -f|--force)
      force="true"
      shift
      ;;
    -n|--no-tags)
      no_tags="true"
      shift
      ;;
    -l|--local)
      local_build="true"
      shift
      ;;
    -i|--image)
      image_name="${2}"
      shift
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

branch=$(git rev-parse --abbrev-ref HEAD)
image_version="$(date +%s)"
package_version="$("${mydir}/get-package-version.sh")"
releases=""

build_and_tag_images() {
  cd "${mydir}/.."

  if [ "${local_build:-}" = "true" ]; then
    log "Building Docker toolchain image"
    docker build -q -t hopr-toolchain \
      -f scripts/toolchain/Dockerfile . &

    log "Waiting for toolchain image to finish"
    wait

    if [ -z "${image_name}" ] || [ "${image_name}" = "hoprd" ] || [ "${image_name}" = "pluto-complete" ]; then
      log "Building Docker image hoprd-local"
      docker build -q -t hoprd-local \
        -f packages/hoprd/Dockerfile . &
    fi

    if [ -z "${image_name}" ] || [ "${image_name}" = "hoprd-nat" ]; then
      log "Building Docker image hoprd-nat-local"
      docker build -q -t hoprd-nat-local \
        --build-arg=HOPRD_RELEASE="${image_version}" \
        scripts/nat &
    fi

    if [ -z "${image_name}" ] || [ "${image_name}" = "anvil" ] || [ "${image_name}" = "pluto-complete" ]; then
      log "Building Docker image hopr-anvil-local"
      docker build -t hopr-anvil-local \
        -f packages/ethereum/Dockerfile.anvil . &
    fi

    log "Waiting for Docker builds (part 1) to finish"
    wait

    if [ -z "${image_name}" ] || [ "${image_name}" = "pluto" ] || [ "${image_name}" = "pluto-complete" ]; then
      log "Building Docker image hopr-pluto-local"
      docker build -q -t hopr-pluto-local \
        --build-arg=ANVIL_IMAGE="hopr-anvil-local" \
        --build-arg=HOPRD_IMAGE="hoprd-local" \
        -f scripts/pluto/Dockerfile . &
    fi

    log "Waiting for Docker builds (part 2) to finish"
    wait
  else
    gcloud builds submit --config cloudbuild.yaml \
      --verbosity=info \
      --substitutions=_PACKAGE_VERSION="${package_version}",_IMAGE_VERSION="${image_version}",_RELEASES="${releases}",_NO_TAGS="${no_tags:-}"
  fi
}

for git_ref in $(jq -r "to_entries[] | .value.git_ref" < "${mydir}/../packages/hoprd/releases.json" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    declare additional_releases
    additional_releases="$(jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .key" < "${mydir}/../packages/hoprd/releases.json")"
    # Prepend "staging-" tag prefix, if this is a staging branch
    if [[ "${branch}" =~ staging/.* ]]; then
      additional_releases="staging-${additional_releases//[[:space:]]/" staging-"}"
    fi
    releases="${releases} ${additional_releases}"
  fi
done

if [ -z "${releases}" ] && [ "${force:-}" != "true" ]; then
  # return early if no environments are found for branch
  log "no releases were configured for branch ${branch}"
  exit 1
fi

build_and_tag_images
