#!/bin/bash

echoerr() { echo "$@" 1>&2; }

if [ $(id -u) -ne 0 ]; then 
	echoerr "ERR: Please run as root!"
	exit 1 
fi


if [ $# -le 0 ]; then
	echoerr "ERR: Missing API token argument!"
	exit 1
fi


declare release="budapest"
declare rundir="/var/hoprd"
declare api_token="$1"


cat <<EOF >docker-compose.yaml
version: "3.9"

# Start an internal-only bridge network to simulate NAT (NATwork)
networks:
  natwork:
    driver: bridge

# Starts HOPRD behind NAT
services:
  hoprd-natted:
    
    image: gcr.io/hoprassociation/hoprd:$release

    command: 
      - "--admin"
      - "--adminHost"
      - "0.0.0.0"
      - "--announce"
      - "--healthCheck"
      - "--healthCheckHost"
      - "0.0.0.0"
      - "--identity"
      - "/app/db/.hopr-identity"
      - "--init"
      - "--rest"
      - "--restHost"
      - "0.0.0.0"
      - "--environment"
      - "$release"
      - "--apiToken"
      - "$api_token"
      - "--password"
      - "pw14775124087585pw"
   
    volumes:
      - "$rundir:/app/db"

    environment:
      - "DEBUG=hopr*"

    networks:
      - "natwork"

    ports:
      - "3000:3000"
      - "3001:3001"
EOF


# Start Docker compose
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$PWD:$PWD" -w="$PWD" docker/compose:1.29.2 up --force-recreate --abort-on-container-exit --exit-code-from hoprd-natted
ec=$?

echo "Node exitted with code $ec"
rm docker-compose.yaml

exit $ec
 