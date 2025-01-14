#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <head_branch>"
  echo ""
  echo "$0 bot/close-2.2.0-rc.1"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && {
  usage
  exit 0
}

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
: ${GH_TOKEN?"environment variable must be set"}
head_branch=${1}
workflow_run_id=$(gh api repos/hoprnet/hoprnet/actions/workflows/build.yaml/runs | jq --arg head_branch "$head_branch" '[.workflow_runs[] | select(.head_branch == $head_branch and .conclusion == "success" and .status == "completed")] | first | .id')
artifacts=$(gh api repos/hoprnet/hoprnet/actions/runs/${workflow_run_id}/artifacts | jq -r '.artifacts[] | "\(.name) \(.archive_download_url)"')
rm -rf ./binaries && mkdir -p ./binaries
while IFS= read -r line; do
  artifact_name=$(echo $line | awk '{print $1}')
  artifact_url=$(echo $line | awk '{print $2}')
  if ! curl -L -s -f -o "binaries/${artifact_name}.zip" -H "Authorization: Bearer ${GH_TOKEN}" "${artifact_url}"; then {
    echo "Error: Failed to download binary file ${artifact_name}"
    exit 1
  } else {
    echo "Downloaded binary file ${artifact_name}..."
  }

done <<<"$artifacts"
