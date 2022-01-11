#!/usr/bin/env sh

if [ $(id -u) -ne 0 ] ; then
  >&2 echo "ERROR: Must run as root"
  exit 1
fi

if [ -z "$HOPR_RELEASE" ]; then
  >&2 echo "ERROR: Specify HOPR_RELEASE environment variable"
  exit 1
fi

if [ ! -f "/var/run/docker.sock" ]; then
  >&2 echo "ERROR: /var/run/docker.sock must be mounted"
  exit 1
fi

readonly network_name="hopr-nat"

if [ "$(docker network ls | grep -c "$network_name" )" = "0" ] ; then
  if ! docker network create -d bridge hopr-nat > /dev/null 2>&1 ; then
    >&2 echo "ERROR: Failed to create network for NAT"
    exit 1
  fi
fi

# Fork here and pass all the environment variables down into the forked image
env > env.list
docker run --pull always -v /var/hoprd/:/app/db -p 3000:3000 -p 3001:3001 \
 --network=hopr-nat \
 --env-file ./env.list \
 "gcr.io/hoprassociation/hoprd:$HOPR_RELEASE" "$@"
