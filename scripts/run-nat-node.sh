#!/bin/bash

function echoerr() { echo "$@" 1>&2; }

if [ $(id -u) -ne 0 ]; then 
  echoerr "ERR: Please run as root!"
  exit 1 
fi


declare rundir="/var/hoprd"
unset RELEASE API_TOKEN

function usage {
  cat <<EOF
Usage: $(basename "$0") [OPTION]

  -r VALUE    relase name, e.g. prague
  -t VALUE    API token
  -h          display help
EOF

  exit 2
}


while getopts ":r:th" optKey; do
  case "$optKey" in
    r)
      RELEASE=$OPTARG
      ;;
    t)
      API_TOKEN=$OPTARG
      ;;
    h|*)
      usage
      ;;
  esac
done

shift $((OPTIND - 1))

[ -z "$RELEASE" ] && usage
[ -z "$API_TOKEN" ] && usage


cat <<EOF >docker-compose.yaml
version: "3.9"

# Start an internal-only bridge network to simulate NAT (NATwork)
networks:
  natwork:
    driver: bridge

# Starts HOPRD behind NAT
services:
  hoprd-natted:
    
    image: gcr.io/hoprassociation/hoprd:$RELEASE

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
      - "$RELEASE"
      - "--apiToken"
      - "$API_TOKEN"
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

