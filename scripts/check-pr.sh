#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail


# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="check-pr"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"

# prints usage of the script
function usage() {
  msg
  msg "Usage: $0 [-h|--help] [-b|--base-branch base-branch-name] [-hb|--head-branch head-branch-name]"
  msg
  msg "This script check the contents of the PR to determine which workflows need to be triggered"
  msg
}

declare base_branch=""
declare head_branch=""
declare results_file="check_pr.log"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -b|--base-branch)
      shift
      base_branch="${1}"
      shift
      ;;
    -hb|--head-branch)
      shift
      head_branch="${1}"
      shift
      ;;
    *)
      usage
      exit 1
      ;;
  esac
done


# Check wether the pushed commits to the PR involve building docker images
function check_push() {
  rm -rf ${results_file}
  touch ${results_file}
  if [ -z "${base_branch}" ] || [  -z "${head_branch:-}" ]; then
    log "Parameter 'base_branch' and 'head_branch' are required"
    usage
    exit 1
  fi

  echo "Checking pushed changeset from ${head_branch} against ${base_branch}"
  git diff --name-only --diff-filter=ACMRT "${base_branch}" "${head_branch}" > changes.txt
  if grep -E "^(scripts/|.github/workflows/build.yaml|.github/workflows/build-toolchain.yaml|Makefile|package.json|.yarnrc.yml|rust-toolchain.toml|.nvmrc|yarn.lock|Cargo.toml)" changes.txt 1> /dev/null; then
      echo "Changes detected on Toolchain"
      echo "build_toolchain=true" >> ${results_file}
  fi
  if grep  -E "^(.github/workflows/build-hopli.yaml|packages/hopli/|Makefile|rust-toolchain.toml|Cargo.toml)" changes.txt 1> /dev/null; then
      echo "Changes detected on Hopli"
      echo "build_hopli=true" >> ${results_file}
  fi

  if grep -E "^(.github/workflows/build-hoprd.yaml|packages/(hoprd|core|core-ethereum|utils|real|connect)/|Makefile|package.json|.yarnrc.yml|rust-toolchain.toml|.nvmrc|yarn.lock|Cargo.toml)" changes.txt 1> /dev/null; then
      echo "Changes detected on Hoprd"
      echo "build_hoprd=true" >> ${results_file}
  fi
  if grep -E "^(scripts/|.github/workflows/build-anvil.yaml|Makefile|packages/ethereum/)" changes.txt | grep -v .md 1> /dev/null; then
      echo "Changes detected on Anvil"
      echo "build_anvil=true" >> ${results_file}
  fi

  rm changes.txt
}

# Main function
check_push
