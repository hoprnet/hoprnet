#!/usr/bin/env bash

cd /home/vagrant || exit 1
sudo docker-compose down > /dev/null 2>&1
sudo docker-compose up --force-recreate --remove-orphans --build -d
