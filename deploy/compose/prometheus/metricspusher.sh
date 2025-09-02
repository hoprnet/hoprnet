#!/bin/sh

# Validate required environment variables
if [ -z "${HOPRD_API_TOKEN}" ]; then
  echo "Error: HOPRD_API_TOKEN is not set"
  exit 1
fi

if [ -z "${METRICS_PUSH_KEY}" ]; then
  echo "Error: METRICS_PUSH_KEY is not set. Get from Bitwarden secret 'Prometheus Pushgateway Hoprd Node'"
  exit 1
fi

METRICS_PUSH_URL=${1}
if [ -z "${METRICS_PUSH_URL}" ]; then
  echo "Error: METRICS_PUSH_URL argument is required"
  exit 1
fi

# Run the loop
while true; do
  echo Publishing metrics ...
  # Add timeout and retry with backoff
  if ! metrics=$(curl -s --max-time 10 -H "X-Auth-Token: ${HOPRD_API_TOKEN}" "http://hoprd:3001/metrics"); then
    echo "Error: Failed to fetch metrics from Hoprd API"
    sleep 5
    continue
  fi

  # Push metrics with timeout
  if ! echo "${metrics}" | curl -s --max-time 10 -u "${METRICS_PUSH_KEY}" --data-binary @- "${METRICS_PUSH_URL}"; then
    echo "Error: Failed to push metrics to ${METRICS_PUSH_URL}"
  fi
  sleep 15
done
