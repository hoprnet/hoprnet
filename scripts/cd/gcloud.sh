#!/bin/bash
set -e #u
shopt -s expand_aliases

# ------ GCloud utilities ------
#
# NB. functions here should not rely on any external env. variables, or functions
# not in this file, as this is intended for reuse in various scenarios.


# $1 - vm name 
# $2 - docker image
gcloud_stop() {
  gcloud compute ssh $ZONE $1 \
    -- "export DOCKER_IMAGE=$2 && docker stop $(docker ps -q --filter \"ancestor=$DOCKER_IMAGE\")"
}
