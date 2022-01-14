#!/usr/bin/env bash

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

declare admin_port=${HOPR_ADMIN_PORT:-3000}
declare rest_port=${HOPR_REST_PORT:-3001}
declare healthcheck_port=${HOPR_HEALTHCHECK_PORT:-8080}
declare network_name="hopr-nat"

if [ "$(docker network ls | grep -c "$network_name" )" = "0" ] ; then
  if ! docker network create -d bridge hopr-nat > /dev/null 2>&1 ; then
    >&2 echo "ERROR: Failed to create network for NAT"
    exit 1
  fi
fi

# Fork here and pass all the environment variables down into the forked image
docker run --pull always -v /var/hoprd/:/app/db -p $admin_port:3000 -p $rest_port:3001 -p $healthcheck_port:8080 \
 --network=hopr-nat \
 --env-file <(env) \
 "gcr.io/hoprassociation/hoprd:$HOPR_RELEASE" "$@"
