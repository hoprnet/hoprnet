#!/usr/bin/env bash

endpoints=("a", "b", "c", "d", "e")

for (( i = 0; i < ${#endpoints[*]}; ++ i )); do
  endpoint=${endpoints[$i]}
  other_endpoint=${endpoints[$i+1]}

  if [ -n "${other_endpoint}" ]; then
    printf $endpoint
    printf $other_endpoint
  fi
done