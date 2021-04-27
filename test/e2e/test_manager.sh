#!/bin/bash
set -e
shopt -s expand_aliases

# Entrypoint for all our end-to-end tests
# @param $1 - Number of the test to run
run_test() {
  if [ -z "$1" ]; then
    echo "Usage: ./test/e2e/run_test.sh [NUMBER]"
    exit 1
  fi
  echo "🧠 Starting test manager, for test $1"
  local TEST="$(ls -h test/e2e/ | grep $1)"
  if [ -z ${TEST} ]; then
    echo "Test $1 not found, try another number"
    exit 1
  fi
  echo "🧠 Running TEST: $TEST"
  DIRNAME=`dirname "$0"`
  source "$DIRNAME/$TEST"
  main
}

run_test $1