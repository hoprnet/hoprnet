#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare script_dir
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

registry=${1?"Usage: $0 <registry>"}

date=`date +%Y-%m-%d'T'%H:%M'Z' -d "60 days ago"`
images=('hopli' 'hoprd')

for image in "${images[@]}"; do
  readarray -t old_pr_image_tags < <(gcloud artifacts docker images list --include-tags ${registry}/${image} --format="json" 2> /dev/null | jq -r --arg date "$date"  '.[]  | select(.updateTime < $date) | select(.tags[] | match("-commit.")).version')
  readarray -t old_untagged_images < <(gcloud artifacts docker images list --include-tags ${registry}/${image} --format="json" 2> /dev/null | jq -r --arg date "$date"  '.[]  | select(.updateTime < $date) | select(.tags | length == 0).version')
  for old_image_tag in "${old_pr_image_tags[@]}" "${old_untagged_images[@]}"; do
    echo "Deleting image ${registry}/${image}@${old_image_tag}"
    gcloud artifacts docker images delete --quiet --delete-tags --async ${registry}/${image}@${old_image_tag}
  done
done
