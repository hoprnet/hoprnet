#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap cleanup EXIT

cleanup() {
  # Kill the ganache instance that we started (if we started one and if it's still running).
  if [ -n "$ganache_pid" ] && ps -p $ganache_pid > /dev/null; then
    kill -9 $ganache_pid
  fi
}

if [ "$SOLIDITY_COVERAGE" = true ]; then
  ganache_port=8555
else
  ganache_port=9545
fi

ganache_running() {
  nc -z localhost "$ganache_port"
}

start_ganache() {
  # 10 accounts with balance 1M ether, needed for high-value tests.
  # taken from https://github.com/hoprnet/hopr-demo-seeds
  local accounts=(
    --account="0xb3faf133044aebecbd8871a920818783f8e3e809a425f131046492925110ebc0,1000000000000000000000000"
    --account="0x5cb8ef3621d94ddd2a9273d00879f74d169677ae8d3ac556dc2c8d194e7d85c8,1000000000000000000000000"
    --account="0x06416989d7877581a37ef97e0e27527b1139f6f63a42c057e60f8c45aa5011a9,1000000000000000000000000"
    --account="0x719b8cebb72977cf49bf9ab0a3bb83fc172820d3cbb4e88e6f858479f6c1e4d4,1000000000000000000000000"
    --account="0x1f4bcb50fb748d3de330105cb21e504bed0b764acf6187cae2be80b0dfb1d1be,1000000000000000000000000"
    --account="0x5e888871b75a05c166402b334ae779bca0ea6273749a24dc34051b668dfde9a9,1000000000000000000000000"
    --account="0x42b0a5265723c40624590dafe20a7bd6477702cf1f65e948060f54404a3c7b5c,1000000000000000000000000"
    --account="0x9d59b34814c2aed4ebd7b3410e7d3aaacc3ddeafee1efa14f8104a8fc47402fe,1000000000000000000000000"
    --account="0xf116b2a53c5309de9f7daf424dc7bb5251fbc0975671e60be3e05e1abdfe776b,1000000000000000000000000"
    --account="0x1148c9aa73c18e1369cbca1ef4ad9e466edb5f883acd57d3a517495adfdf4631,1000000000000000000000000"
  )

  if [ "$SOLIDITY_COVERAGE" = true ]; then
    npx ganache-cli-coverage --emitFreeLogs true --allowUnlimitedContractSize true --gasLimit 0xfffffffffffff --port "$ganache_port" "${accounts[@]}" > /dev/null &
  elif [ "$ONLY_NETWORK" = true ]; then
    npx ganache-cli --gasLimit 0xfffffffffff --port "$ganache_port" "${accounts[@]}"
  else
    npx ganache-cli --gasLimit 0xfffffffffff --port "$ganache_port" "${accounts[@]}" > /dev/null &
  fi

  ganache_pid=$!

  echo "Waiting for ganache to launch on port "$ganache_port"..."

  while ! ganache_running; do
    sleep 0.1 # wait for 1/10 of the second before check again
  done

  echo "Ganache launched!"
}

if ganache_running; then
  echo "Using existing ganache instance"
else
  echo "Starting our own ganache instance"
  start_ganache
fi

npx truffle version

if [ "$SOLIDITY_COVERAGE" = true ]; then
  npx solidity-coverage
elif [ "$ONLY_NETWORK" = true ]; then
  echo "Network ready!"
else
  npx truffle test --network test --debug "$@"
fi