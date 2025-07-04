#!/bin/bash
set -Eeo pipefail

deb-systemd-helper disable hoprd.service >/dev/null
deb-systemd-helper stop hoprd.service >/dev/null
deb-systemd-helper daemon-reexec
deb-systemd-helper daemon-reload
