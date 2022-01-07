#!/usr/bin/env bash

set -Eeuo pipefail

if [[ $(id -u) -ne 0 ]] ; then
  >&2 echo "ERROR: Must run as root!"
  exit 1
fi

if [[ $# -lt 1 ]]; then
  >&2 echo "ERROR: Must specify release!"
  exit 1
fi

readonly release=$1
shift

echo "Starting HOPR release $release behind NAT..."

docker network create -d bridge hopr-nat
docker run --pull always -v /var/hoprd/:/app/db -p 3000:3000 -p 3001:3001 \
 -e "DEBUG=hopr*,-hopr-connect*" -e "GCLOUD=1" \
 --network=hopr-nat "gcr.io/hoprassociation/hoprd:$release" "$@"
