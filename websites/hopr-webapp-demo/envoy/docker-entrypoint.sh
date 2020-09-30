#!/bin/sh
set -e

echo "Generating envoy.yaml config file..."
mkdir -p /app/envoy
cat /tmpl/envoy.yaml.tmpl | envsubst \$SERVICE_ADDRESS > /app/envoy/envoy.yaml

echo "Starting Envoy..."
envoy -c /app/envoy/envoy.yaml