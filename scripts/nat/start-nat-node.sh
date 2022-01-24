#!/usr/bin/env bash

# This is a thin wrapper script for the Docker HOPRd container
# to start in a different network on GCloud.
# Per default, the Docker containers in GCloud VMs are started in with --network=host and this cannot
# be changed. To simulate a node behind NAT, this script creates a dedicated bridge network, and then starting
# the HOPRd Docker image bound to that network. The P2P port 9091 is intentionally not exposed.

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

if [[ $# = 1 && "$1" = "--version" ]]; then
  docker run --pull always --rm "gcr.io/hoprassociation/hoprd:$HOPRD_RELEASE" --version 2> /dev/null
  exit $?
fi

# Default hoprd ports, configurable using env variables passed to the container
declare admin_port=${HOPRD_ADMIN_PORT:-3000}
declare rest_port=${HOPRD_REST_PORT:-3001}
declare healthcheck_port=${HOPRD_HEALTHCHECK_PORT:-8080}

declare container_name=${HOPRD_CONTAINER_NAME:-hoprd-behind-nat}
declare network_name="hopr-nat"

# Create an isolated network to force NAT
if [ "$(docker network ls | grep -c "${network_name}" )" = "0" ] ; then
  if ! docker network create -d bridge ${network_name} > /dev/null 2>&1 ; then
    >&2 echo "ERROR: Failed to create network for NAT"
    exit 1
  fi
fi

# Stop & remove the Docker container if it was running already
docker stop ${container_name} > /dev/null 2>&1 || true && docker rm ${container_name} > /dev/null 2>&1 || true

# Fork here and pass all the environment variables down into the forked image
docker run -d --pull always -v /var/hoprd/:/app/db -p ${admin_port}:3000 -p ${rest_port}:3001 -p ${healthcheck_port}:8080 \
 --name=${container_name} --restart=on-failure \
 --network=${network_name} \
 --env-file <(env) \
 "gcr.io/hoprassociation/hoprd:$HOPRD_RELEASE" "$@"
