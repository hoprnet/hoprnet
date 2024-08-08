#!/bin/bash

# Check if contracts-addresses.json has been modified
if git diff --cached --name-only | grep -q 'ethereum/contracts/contracts-addresses.json'; then
  echo "Error: Changes to 'ethereum/contracts/contracts-addresses.json' are not allowed."
  exit 1
fi
