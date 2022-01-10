#!/usr/bin/env sh

if [ $(id -u) -ne 0 ] ; then
  >&2 echo "ERROR: Must run as root!"
  exit 1
fi

if [ -z "$HOPR_RELEASE" ]; then
  >&2 echo "ERROR: Specify HOPR_RELEASE environment variable"
  exit 1
fi

#echo "Starting HOPR release '$HOPR_RELEASE' behind NAT..."

readonly network_name="hopr-nat"

if [ "$(docker network ls | grep -c "$network_name" )" = "0" ]; then
  docker network create -d bridge hopr-nat
fi

docker run --pull always -v /var/hoprd/:/app/db -p 3000:3000 -p 3001:3001 \
 -e "DEBUG=hopr*,-hopr-connect*" -e "GCLOUD=1" \
 --network=hopr-nat "gcr.io/hoprassociation/hoprd:$HOPR_RELEASE" "$@"
