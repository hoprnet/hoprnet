#!/bin/bash
set -e #u
shopt -s expand_aliases
#set -o xtrace

source scripts/cd/start-bootstrap-server.sh

# ---- On Deployment -----
#
# This is run on pushes to master, or release/**
#

# -- Setup Dependencies --
ethers --version || npm install -g @ethersproject/cli

# Get version from package.json
RELEASE=$(node -p -e "require('./packages/hoprd/package.json').version")

start_bootstrap
