#!/bin/sh

systemctl disable hoprd.service
systemctl stop hoprd.service
systemctl daemon-reexec
systemctl daemon-reload