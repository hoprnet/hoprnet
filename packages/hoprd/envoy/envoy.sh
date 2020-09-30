#!/bin/sh
set -e

echo "Starting Envoy..."
envoy -c /app/envoy/envoy.yaml
