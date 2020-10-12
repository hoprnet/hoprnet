#!/bin/bash
# When we publish new versions of packages in the npm registry,
# it may take up to 15 minutes for the packages to be available.
# This utility exits once the packages are available, or a deadline is reached.
# Usage: bash wait-npm-publish.sh <version> <deadline_in_secs> <wait_time_for_each_check>
# Example: `wait-npm-publish.sh 1.17.0-alpha.62`

# ex: 1.17.0-alpha.50
expected_version=${1}
# maximum time to wait for, in seconds
deadline_secs=${2:-900}
# time to wait before every check, in seconds
wait_secs=${3:-60}

start_time=$(date +%s)
deadline=$(( $start_time + $deadline_secs ))
# get list of packages that were published
packages=$(../node_modules/.bin/lerna list)

echo "Searching for packages in version ${expected_version}."
echo "Running with:"
echo "  expected_version  = ${expected_version}"
echo "  deadline_secs     = ${deadline_secs}"
echo "  wait_secs         = ${wait_secs}"

found_all=false

# loop until all packages are publish or we reach deadline
while true; do
  sleep $wait_secs

  for package in $packages; do
    search="${package}@${expected_version}"
    result=$(npm show $search version)

    if [ "$result" = "" ]; then
      echo "Missing ${package}!"
      break
    else
      echo "Found ${package}."
    fi

    found_all=true
  done

  # all packages were found
  if $found_all; then
    echo "Found all packages!"
    exit 0;
  fi

  # deadline reached
  now=$(date +%s)
  if [ "$deadline" -lt "$now" ]; then
    echo "Deadline reached, not all packages were published."
    exit 1;
  fi

  echo "Rechecking in ${wait_secs} seconds."
done
