#!/usr/bin/env bash

set -e

which websocat > /dev/null || \
(
    sudo apt-get update && \
    sudo apt-get install curl -y && \
    curl -sLO https://github.com/vi/websocat/releases/download/v1.8.0/websocat_1.8.0_newer_amd64.deb && \
    sudo dpkg -i websocat_1.8.0_newer_amd64.deb
)
