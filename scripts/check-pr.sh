#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail


# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="check-pr"
source "${mydir}/utils.sh"

# prints usage of the script
function usage() {
  msg
  msg "Usage: $0 [-h|--help] [-e|--event event-name] [-l|--labels label-name] [-b|--base-branch base-branch-name]"
  msg
  msg "This script check the contents of the PR to determine which workflows need to be triggered"
  msg
}


declare event_type=""
declare labels=""
declare base_branch=""
declare results_file="check_pr_results.txt"

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
    -l|--labels)
      shift
      labels="${1}"
      shift
      ;;
    -b|--base-branch)
      shift
      base_branch="${1}"
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

if [ -z "${event_type}" ]; then
  # return early if required parameters have been set
  log "Parameter 'event' is required"
  usage
  exit 1
fi

if [ ! -z "${base_branch}" ] && [ ! -z "${labels:-}" ]; then
  # return early if required parameters have been set
  log "Parameter 'base_branch' or 'labels' are required"
  usage
  exit 1
fi

# Check wether the pushed commits to the PR involve building docker images
function check_push() {
  declare head_branch=`git rev-parse --abbrev-ref HEAD`
  
  echo "Checking pushed changeset from ${head_branch} against ${base_branch}"
  git diff --name-only --diff-filter=ACMRT ${base_branch} ${head_branch} > changes.txt
  if cat changes.txt | grep -e ^scripts/ -e ^Makefile$ -e ^package.json$ -e ^.yarnrc.yml$ -e ^rust-toolchain.toml$ -e ^.nvmrc -e ^yarn.lock$ -e ^Cargo.toml 1> /dev/null; then
      echo "Changes detected on Toolchain"
      echo "toolchain=true" >> ${results_file}
  fi
  if cat changes.txt | grep ^packages/hopli/ 1> /dev/null; then
      echo "Changes detected on Hopli"
      echo "hopli=true" >> ${results_file}
  fi

  if cat changes.txt | grep -v ^packages/hopli/ | grep -v ^scripts | grep -v ^.processes | grep -v ^docs/ | grep -v .md 1> /dev/null; then
      echo "Changes detected on Hoprd"
      echo "hoprd=true" >> ${results_file}
  fi
  rm changes.txt
}

# Check how to react against the new labels added
function check_labeled() {
  declare new_label=""
  new_label="${1}"
  echo "Checking adding of label ${new_label}"
}

# Check how to react against the labels removed
function check_unlabeled() {
  declare removed_label=""
  removed_label="${1}"
  echo "Checking removal of label ${removed_label}"
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
      check_unlabeled labels[0]
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main