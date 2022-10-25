#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare -x HOPR_LOG_ID="deployment-status"
source "${mydir}/gcloud.sh"

readonly flag_prefix="DEPLOYMENT_ACTIVE"

# Use current branch if not specified as argument
current_branch=$(git rev-parse --abbrev-ref HEAD)
used_branch=${2:-$current_branch}
current_deployment=$(echo "${used_branch}" | sed 's|/|_|g' | tr '[:lower:]' '[:upper:]')

msg "Deployment flag name for the branch \"${used_branch}\": ${current_deployment}"

usage() {
   msg
   msg "Usage: $0 [-h|--help] [-a|--activate] [-d|--deactivate] [-c|--check] [branch]"
   msg
   msg "This script can check or manipulate the status flag of the Google Cloud nodes deployment"
   msg "from the current Git branch. When the flag is set, the deployment is considered active."
}

activate() {
  local ts="$(date +%s)"
  gcloud_set_project_flag "${flag_prefix}_${current_deployment}" "${ts}"
  msg "Set active deployment flag ${current_deployment}"
}

deactivate() {
  gcloud_unset_project_flag "${flag_prefix}_${current_deployment}"
  msg "Removed active deployment flag ${current_deployment}"
}

check() {
  local is_set="$(gcloud_isset_project_flag "${flag_prefix}_${current_deployment}")"
  local ec=0
  if [ "${is_set,,}" = "true" ]; then
    msg "✅ Active deployment flag ${current_deployment} IS set"
  else
    msg "❌ Active deployment flag ${current_deployment} IS NOT set"
    ec=1
  fi

  return ${ec}
}

if [ $# -le 1 ]; then
  usage
  exit 1
fi

if [ "${1}" = "activate" ]; then
  activate
fi

if [ "${1}" = "deactivate" ]; then
  deactivate
fi

if [ "${1}" = "check" ]; then
  check
  exit $?
fi

