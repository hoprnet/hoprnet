#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["google-cloud-artifact-registry==1.15.2","google-auth==2.38.0"]
# ///

import sys
from datetime import datetime, timedelta
from google.cloud import artifactregistry_v1
from google.auth.exceptions import DefaultCredentialsError
import argparse

# Ensure the script exits on errors
def list_docker_images(client, parent):
    try:
        request = artifactregistry_v1.ListDockerImagesRequest(parent=parent)
        return [image for image in client.list_docker_images(request=request)]
    except Exception as e:
        print(f"Error listing docker images: {str(e)}", file=sys.stderr)
        sys.exit(1)

def delete_docker_image(client, name, dry_run):
    if dry_run:
        print(f"Dry-run mode: Would delete image {name}")
        return
    try:
        request = artifactregistry_v1.DeleteDockerImageRequest(name=name)
        client.delete_docker_image(request=request)
    except Exception as e:
        print(f"Error deleting docker image: {str(e)}", file=sys.stderr)
        sys.exit(1)

# Parse command-line arguments
parser = argparse.ArgumentParser(description="Cleanup old Docker images.")
parser.add_argument("registry", help="Docker image registry")
parser.add_argument("-n", "--dry-run", action="store_true", help="Simulate the deletion without making any changes")
parser.add_argument("-d","--days", type=int, default=60, help="Number of days to consider an image old (default: 60)")
args = parser.parse_args()

registry = args.registry
dry_run = args.dry_run
days = args.days
date = (datetime.utcnow() - timedelta(days=days)).strftime('%Y-%m-%dT%H:%MZ')
images = ['hopli', 'hoprd']

try:
    client = artifactregistry_v1.ArtifactRegistryClient()
except DefaultCredentialsError as e:
    print(f"Error with credentials: {str(e)}", file=sys.stderr)
    sys.exit(1)

project = registry.split('/')[1]
location = registry.split('.')[0].replace('-docker', '')

for image in images:
    parent = f"projects/{project}/locations/{location}/repositories/{image}"
    docker_images = list_docker_images(client, parent)
    old_pr_image_tags = [img.name for img in docker_images if img.update_time < date and any('commit' in tag for tag in img.tags)]
    old_untagged_images = [img.name for img in docker_images if img.update_time < date and len(img.tags) == 0]

    old_image_tags = old_pr_image_tags + old_untagged_images

    for old_image_tag in old_image_tags:
        print(f"Deleting image {old_image_tag}")
        delete_docker_image(client, old_image_tag, dry_run)
