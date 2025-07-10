#!/bin/bash
set -Eeuo pipefail

systemctl disable hoprd.service >/dev/null
systemctl stop hoprd.service >/dev/null
systemctl daemon-reexec
systemctl daemon-reload
