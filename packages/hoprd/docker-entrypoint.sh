#!/bin/bash
set -e

# if no arguments passed on runtime, start in default mode
if [ "$#" -eq 0 ]; then
  exec pm2-runtime process.yaml \
    --password="open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0" \
    --env "replaced_by_dockerfile_network" \
    --admin
else
  exec pm2-runtime process.yaml "${@:1}"
fi
