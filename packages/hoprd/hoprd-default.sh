#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

DEBUG="hopr*" hoprd --password="" --init --admin
