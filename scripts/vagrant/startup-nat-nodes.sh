#!/usr/bin/env bash

cd /opt/hopr && yarn && yarn build
cd /home/vagrant && docker-compose up -d --force-recreate --remove-orphans
