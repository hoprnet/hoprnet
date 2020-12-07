#!/bin/bash
# @param {('refs/heads/master'|'refs/heads/cd/**')} GITHUB_REF - The reference from a github branch 
set -e

# @TODO: Automate IP creating when not found.
# @TODO: Move this to an env file or secret.
declare -A environments=(
  ["debug"]="34.65.56.229"
  ["master"]="34.65.102.152"
  ["luzern"]="34.65.5.42"
  ["zug"]="34.65.158.118"
  ["larnaca"]="34.65.66.167"
)

if [[ -z "${GITHUB_REF}" ]]; then
  REF="master"
else
  REF="${GITHUB_REF}"
fi

RELEASE_NAME=$(echo "$REF" | sed -e "s#refs/heads/##g") # Removing `refs/heads`
RELEASE_NAME=$(echo "$RELEASE_NAME" | sed -e "s#cd/##g") # Removing `cd`

declare RELEASE_IP=${environments[$RELEASE_NAME]}

echo "RELEASE_NAME=$RELEASE_NAME"
echo "RELEASE_IP=$RELEASE_IP"


