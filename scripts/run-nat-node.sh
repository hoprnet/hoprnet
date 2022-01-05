#!/usr/bin/env bash

set -e
set -u
set -o pipefail

function echoerr() { echo "$@" 1>&2; }

if [ $(id -u) -ne 0 ]; then 
  echoerr "ERR: Please run as root!"
  exit 1 
fi

unset RELEASE API_TOKEN DBG_STRING HOPR_VOLUME
GENERATE_ONLY=false
RELEASE=""
API_TOKEN=""
DBG_STRING=""
HOPR_VOLUME="/var/hoprd"

function usage {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

  -r VALUE    relase name, e.g. prague (required)
  -t VALUE    API token (required)
  -d VALUE    debug environment variable (DEBUG=VALUE, default: none)
  -v VALUE    hoprd database directory mount (default: /var/hoprd)
  -g          only generate docker-compose.yaml, do not start docker compose
  -h          display help
EOF

  exit 2
}


while getopts "r:t:d:v:gh" optKey; do
  case "$optKey" in
    r)
      RELEASE=$OPTARG
      ;;
    t)
      API_TOKEN=$OPTARG
      ;;
    d)
      DBG_STRING=$OPTARG
      ;;
    v)
      HOPR_VOLUME=$OPTARG
      ;;
    g)
      GENERATE_ONLY=true
      ;;
    h|*)
      usage
      ;;
  esac
done

shift $((OPTIND - 1))

# Required parameters
[ -z "$RELEASE" ] && usage
[ -z "$API_TOKEN" ] && usage

# Optional parameters
[ -z "$HOPR_VOLUME" ] && HOPR_VOLUME="/var/hoprd"


cat <<EOF >docker-compose.yaml
version: "3.9"

# Start an internal-only bridge network to simulate NAT (NATwork)
networks:
  hopr-local-network:
    driver: bridge

# Starts HOPRD behind NAT
services:
  hoprd-nat:
    
    image: gcr.io/hoprassociation/hoprd:$RELEASE

    command: 
      - "--admin"
      - "--adminHost"
      - "0.0.0.0"
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
      - "$HOPR_VOLUME:/app/db"

    networks:
      - "hopr-local-network"

    ports:
      - "3010:3000"
      - "3011:3001"
EOF


# Add environment variables for debugging
if [ ! -z "$DBG_STRING" ]; then

cat <<EOF >>docker-compose.yaml
    environment:
      - "DEBUG=$DBG_STRING"
EOF

fi


# Start Docker compose
[ "$GENERATE_ONLY" = false ] && docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$PWD:$PWD" -w="$PWD" docker/compose:1.29.2 up -d --force-recreate --remove-orphans


