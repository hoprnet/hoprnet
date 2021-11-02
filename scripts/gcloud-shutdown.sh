#!/usr/bin/env bash
ZONE=$(gcloud compute instances list --filter=$(hostname) --format 'csv[no-heading](zone)')
gcloud compute instances delete $(hostname) --quiet --zone="$ZONE"