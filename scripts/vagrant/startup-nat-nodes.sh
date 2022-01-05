#!/usr/bin/env bash

cd /opt/hopr && yarn && yarn build

cd /home/vagrant && sudo docker-compose up -d --force-recreate --remove-orphans
