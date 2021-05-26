#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <package-name> <package-version>" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
[ -z "${1:-}" ] && { echo "Missing parameter <package-name>" >&2; usage; exit 1; }
[ -z "${2:-}" ] && { echo "Missing parameter <package-version>" >&2; usage; exit 1; }

declare package_name
declare package_version

package_name="$1"
package_version="$2"

# do work

npm view "@hoprnet/${package_name}@${package_version}" --json
