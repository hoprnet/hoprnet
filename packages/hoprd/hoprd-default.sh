#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

DEBUG="hopr*,libp2p:mplex:stream" hoprd --password="" --init --rest --admin
