#!/bin/bash
set -euo pipefail

# Default target directory can be overridden by setting TARGET_DIR
TARGET_DIR="${TARGET_DIR:-/tmp/hoprd-sdk-python}"

# Check for required commands
command -v swagger-codegen3 >/dev/null 2>&1 || {
  echo "swagger-codegen3 is required but not installed" >&2
  exit 1
}

# Use mktemp for secure temporary file creation
config_file=$(mktemp)
trap 'rm -f "${config_file}"' EXIT

# Generate config file
echo '{"packageName":"hoprd_sdk","projectName":"hoprd-sdk","packageVersion":"'"$(./scripts/get-current-version.sh docker)"'","packageUrl":""}' >"${config_file}"

# Ensure target directory exists and is empty
mkdir -p "${TARGET_DIR}"
rm -rf "${TARGET_DIR}"/*

# create the openapi.spec.json
hoprd-api-schema >|/tmp/openapi.spec.json

# Generate SDK
if ! swagger-codegen3 generate \
  -l python \
  -o "${TARGET_DIR}" \
  -i /tmp/openapi.spec.json \
  -c "${config_file}"; 
then
  echo "Failed to generate SDK" >&2
  exit 1
fi

echo "SDK generated successfully in ${TARGET_DIR}"
