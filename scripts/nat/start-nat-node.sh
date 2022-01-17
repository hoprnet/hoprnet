#!/usr/bin/env bash

if [ $(id -u) -ne 0 ] ; then
  >&2 echo "ERROR: Must run as root"
  exit 1
fi

if [ -z "$HOPRD_RELEASE" ]; then
  >&2 echo "ERROR: Must specify HOPRD_RELEASE environment variable"
  exit 1
fi

if [ ! -S "/var/run/docker.sock" ]; then
  >&2 echo "ERROR: /var/run/docker.sock must be mounted"
  exit 1
fi

# Default hoprd ports, configurable using env variables passed to the container
declare admin_port=${HOPRD_ADMIN_PORT:-3000}
declare rest_port=${HOPRD_REST_PORT:-3001}
declare healthcheck_port=${HOPRD_HEALTHCHECK_PORT:-8080}

declare network_name="hopr-nat"

if [ "$(docker network ls | grep -c "${network_name}" )" = "0" ] ; then
  if ! docker network create -d bridge ${network_name} > /dev/null 2>&1 ; then
    >&2 echo "ERROR: Failed to create network for NAT"
    exit 1
  fi
fi

# Fork here and pass all the environment variables down into the forked image
# NOTE: The lib2p2 port is not published to prevent bypassing NAT from the outside
docker run --pull always -v /var/hoprd/:/app/db -p ${admin_port}:3000 -p ${rest_port}:3001 -p ${healthcheck_port}:8080 \
 --network=${network_name} \
 --env-file <(env) \
 "gcr.io/hoprassociation/hoprd:$HOPRD_RELEASE" "$@"
