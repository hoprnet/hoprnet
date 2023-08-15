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
# shellcheck disable=SC1090
source "${mydir}/gcloud.sh"

readonly flag_prefix="DEPLOYMENT_FAILED"

# Use current branch if not specified as argument
current_branch=$(git rev-parse --abbrev-ref HEAD)
used_branch=${2:-$current_branch}
current_deployment=$(echo "${used_branch}" | sed 's|/|_|g' | tr '[:lower:]' '[:upper:]')

msg "Use flag name for the branch \"${used_branch}\": ${flag_prefix}_${current_deployment}"

usage() {
   msg
   msg "Usage: $0 <set|unset|check> [branch]"
   msg
   msg "This script can check or manipulate the failure flag of the Google Cloud nodes deployment"
   msg "from the given Git branch. When the flag is set, the deployment is considered in a failed state."
   msg "If no branch argument is given, the current active branch is used."
}

set_fail_flag() {
  local ts="$(date +%s)"
  gcloud_set_project_flag "${flag_prefix}_${current_deployment}" "${ts}"
  msg "Set deployment failure flag ${current_deployment}"
}

unset_fail_flag() {
  gcloud_unset_project_flag "${flag_prefix}_${current_deployment}"
  msg "Removed deployment failure flag ${current_deployment}"
}

check() {
  local is_set="$(gcloud_isset_project_flag "${flag_prefix}_${current_deployment}")"
  local ec=0
  if [ "${is_set,,}" = "true" ]; then
    msg "❌ Deployment failure flag ${current_deployment} IS set"
    ec=1
  else
    msg "✅ Deployment failure flag ${current_deployment} NOT set"
  fi

  return ${ec}
}

if [ $# -lt 1 ]; then
  usage
  exit 1
fi

if [ "${1}" = "set" ]; then
  set_fail_flag
fi

if [ "${1}" = "unset" ]; then
  unset_fail_flag
fi

if [ "${1}" = "check" ]; then
  check
  exit $?
fi

