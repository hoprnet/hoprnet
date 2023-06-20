#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail


# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="check-pr"
# shellcheck source=scripts/utils.sh
source "${mydir}/utils.sh"

# prints usage of the script
function usage() {
  msg
  msg "Usage: $0 [-h|--help] [-e|--event event-name] [-l|--label label-name] [-b|--base-branch base-branch-name] [-hb|--head-branch head-branch-name]"
  msg
  msg "This script check the contents of the PR to determine which workflows need to be triggered"
  msg
}

declare event_type=""
declare label=""
declare base_branch=""
declare results_file="check_pr.log"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -e|--event)
      shift
      event_type="${1}"
      shift
      ;;
    -l|--label)
      shift
      label="${1}"
      shift
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

if [ -z "${event_type}" ]; then
  # return early if required parameters have been set
  log "Parameter 'event' is required"
  usage
  exit 1
fi

# Check wether the pushed commits to the PR involve building docker images
function check_push() {
  if [ -z "${base_branch}" ] || [  -z "${head_branch:-}" ]; then
    log "Parameter 'base_branch' and 'head_branch' are required"
    usage
    exit 1
  fi
  
  echo "Checking pushed changeset from ${head_branch} against ${base_branch}"
  git diff --name-only --diff-filter=ACMRT "${base_branch}" "${head_branch}" > changes.txt
  if grep -e ^scripts/ -e ^Makefile$ -e ^package.json$ -e ^.yarnrc.yml$ -e ^rust-toolchain.toml$ -e ^.nvmrc -e ^yarn.lock$ -e ^Cargo.toml changes.txt 1> /dev/null; then
      echo "Changes detected on Toolchain"
      echo "build_toolchain=true" >> ${results_file}
  fi
  if grep ^packages/hopli/ changes.txt 1> /dev/null; then
      echo "Changes detected on Hopli"
      echo "build_hopli=true" >> ${results_file}
  fi

  if grep -v ^packages/hopli/ changes.txt | grep -v ^scripts | grep -v ^.processes | grep -v ^docs/ | grep -v .md 1> /dev/null; then
      echo "Changes detected on Hoprd"
      echo "build_hoprd=true" >> ${results_file}
  fi
  if grep -e ^scripts/ -e ^Makefile$ -e ^packages/ethereum/ changes.txt | grep -v .md 1> /dev/null; then
      echo "Changes detected on Anvil"
      echo "build_anvil=true" >> ${results_file}
  fi
  rm changes.txt
}

# Check how to react against the new labels added
function check_labeled() {
  if [ -z "${label:-}" ]; then
    log "Parameter 'label' is required"
    usage
    exit 1
  fi
  echo "Checking adding of label ${label}"
  case "${label}" in
    deploy_nodes)
      echo "create_deployment=true" > ${results_file}
      ;;
    *)
      echo "Skipping any action with the label added: ${label}"
      echo ""  > ${results_file}
      ;;
  esac

}

# Check how to react against the labels removed
function check_unlabeled() {
  if [ -z "${label:-}" ]; then
    log "Parameter 'label' is required"
    usage
    exit 1
  fi
  echo "Checking removal of label ${label}"
  case "${label}" in
    deploy_nodes)
      echo "delete_deployment=true" > ${results_file}
      ;;
    *)
      echo "Skipping any action with the label removed: ${label}"
      echo ""  > ${results_file}
      ;;
  esac
}

# Main function to trigger specific action
function main() {
  rm -rf ${results_file}
  case "${event_type}" in
    push)
      check_push
      ;;
    labeled)
      check_labeled
      ;;
    unlabeled)
      check_unlabeled
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main