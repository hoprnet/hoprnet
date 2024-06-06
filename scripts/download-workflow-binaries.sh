#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <pr_number>"
  echo ""
  echo "$0 6314"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
: ${GH_TOKEN?"environment variable must be set"}
pr_number=${1}
workflow_run_id=$(gh api repos/hoprnet/hoprnet/actions/workflows/build.yaml/runs | jq --argjson pr_number "$pr_number" '[.workflow_runs[] | select(.pull_requests[].number == $pr_number and .conclusion == "success" and .status == "completed" )] | first| .id')
artifacts=$(gh api repos/hoprnet/hoprnet/actions/runs/${workflow_run_id}/artifacts | jq -r '.artifacts[] | "\(.name) \(.archive_download_url)"')
rm -rf ./binaries && mkdir -p ./binaries
while IFS= read -r line; do
  artifact_name=$(echo $line | awk '{print $1}')
  artifact_url=$(echo $line | awk '{print $2}')
  echo "Downloading ${artifact_name}"
  curl -L -s -o "binaries/${artifact_name}.zip" -H "Authorization: Bearer ${GH_TOKEN}" "${artifact_url}"
done <<< "$artifacts"


