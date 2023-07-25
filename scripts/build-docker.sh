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
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 [-h|--help] [-f|--force] [-n|--no-tags] [-i|--image] [-p]"
  msg
  msg "This script builds all docker images."
  msg
  msg "Use -i to build a single image locally instead of all images."
  msg "Supported values for -i are 'hoprd', 'hoprd-nat', 'anvil', 'hopli', pluto', 'pluto-complete'"
  msg
  msg "Use -f to force a Docker builds even though no environment can be found. This is useful for local testing. No additional docker tags will be applied though if no environment has been found which is in contrast to the normal execution of the script."
  msg
  msg "Use -p to use Podman/Buildah instead of Docker"
}

declare image_version package_version releases branch force build_cmd
declare image_name=""
build_cmd="docker"

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
    -i|--image)
      image_name="${2}"
      shift
      shift
      ;;
    -p)
      build_cmd="podman"
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

  log "Building Docker image hopr-toolchain-local:latest"
  ${build_cmd} build -t hopr-toolchain-local:latest \
    -f scripts/toolchain/Dockerfile . &

  log "Waiting for Docker builds (part 1) to finish"
  wait

  if [ -z "${image_name}" ] || \
     [ "${image_name}" = "hoprd" ] || \
     [ "${image_name}" = "hoprd-nat" ] || \
     [ "${image_name}" = "pluto-complete" ]; then
    log "Building Docker image hoprd-local:latest"
    ${build_cmd} build -t hoprd-local:latest \
      --build-arg=HOPR_TOOLCHAIN_IMAGE="hopr-toolchain-local:latest" \
      -f packages/hoprd/Dockerfile . &
  fi

  if [ -z "${image_name}" ] || \
     [ "${image_name}" = "hoprd-nat" ]; then
    log "Building Docker image hoprd-nat-local:latest"
    ${build_cmd} build -t hoprd-nat-local:latest \
      --build-arg=HOPRD_RELEASE="${image_version}" \
      scripts/nat &
  fi

  if [ -z "${image_name}" ] || \
     [ "${image_name}" = "anvil" ] || \
     [ "${image_name}" = "pluto-complete" ]; then
    log "Building Docker image hopr-anvil-local:latest"
    ${build_cmd} build -t hopr-anvil-local:latest \
      --build-arg=HOPR_TOOLCHAIN_IMAGE="hopr-toolchain-local:latest" \
      -f packages/ethereum/Dockerfile.anvil . &
  fi

  if [ -z "${image_name}" ] || \
     [ "${image_name}" = "hopli" ] || \
     [ "${image_name}" = "pluto" ] || \
     [ "${image_name}" = "pluto-complete" ]; then
    log "Building Docker image hopli-local:latest"
    ${build_cmd} build -t hopli-local:latest \
      --build-arg=HOPR_TOOLCHAIN_IMAGE="hopr-toolchain-local:latest" \
      -f packages/hopli/Dockerfile . &
  fi

  log "Waiting for Docker builds (part 2) to finish"
  wait

  if [ -z "${image_name}" ] || \
     [ "${image_name}" = "pluto" ] || \
     [ "${image_name}" = "pluto-complete" ]; then
    log "Building Docker image hopr-pluto-local:latest"
    ${build_cmd} build -t hopr-pluto-local:latest \
      --build-arg=ANVIL_IMAGE="hopr-anvil-local:latest" \
      --build-arg=HOPLI_IMAGE="hopli-local:latest" \
      --build-arg=HOPRD_IMAGE="hoprd-local:latest" \
      -f scripts/pluto/Dockerfile . &
  fi

  log "Waiting for Docker builds (part 3) to finish"
  wait
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
