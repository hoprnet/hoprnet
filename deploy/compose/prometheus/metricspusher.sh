#!/bin/sh


METRICS_PUSH_URL=${1}

# Run the loop
while true; do 
    echo Publishing metrics ...
    curl -s -H "X-Auth-Token: ${HOPRD_API_TOKEN}" http://hoprd:3001/api/v3/node/metrics | \
    curl -s --data-binary @- ${METRICS_PUSH_URL}
    sleep 15
done