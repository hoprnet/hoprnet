#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["google-cloud-artifact-registry==1.15.2","google-auth==2.38.0"]
# ///

import sys
from datetime import datetime, timedelta
from google.cloud import artifactregistry_v1
from google.auth.exceptions import DefaultCredentialsError
import argparse
import re

# Ensure the script exits on errors
def list_docker_images(client, parent):
    """
    Lists Docker images under the specified parent path.

    Args:
        client: Artifact Registry client instance.
        parent: The parent path in the format `projects/{project}/locations/{location}/repositories/{repo}`.

    Returns:
        A list of Docker images.

    Raises:
        SystemExit: If an error occurs while listing Docker images.
    """
    try:
        request = artifactregistry_v1.ListDockerImagesRequest(parent=parent)
        return [image for image in client.list_docker_images(request=request)]
    except Exception as e:
        print(f"Error listing docker images: {str(e)}", file=sys.stderr)
        sys.exit(1)

def delete_docker_image(client, name, dry_run):
    """
    Deletes a Docker image by its name.

    Args:
        client: Artifact Registry client instance.
        name: The name of the Docker image to delete.
        dry_run: If True, simulates the deletion without making changes.

    Raises:
        SystemExit: If an error occurs while deleting the Docker image.
    """
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
parser.add_argument("-d", "--days", type=int, default=60, help="Number of days to consider an image old (default: 60)")
args = parser.parse_args()

# Extract and validate command-line arguments
registry = args.registry
dry_run = args.dry_run
days = args.days
date = datetime.utcnow() - timedelta(days=days)
images = ["hopli", "hoprd"]

# Example registry URL: europe-west3-docker.pkg.dev/my-project/my-repo
registry_pattern = re.compile(r"^(?P<location>[a-z0-9-]+)-docker\.pkg\.dev/(?P<project>[^/]+)/(?P<repo>[^/]+)$")
match = registry_pattern.match(registry)

if not match:
    print(f"Invalid registry format: {registry}", file=sys.stderr)
    sys.exit(1)

# Extract location, project, and repository from the registry URL
location = match.group("location")
project = match.group("project")
repo = match.group("repo")

# Initialize the Artifact Registry client
try:
    client = artifactregistry_v1.ArtifactRegistryClient()
except DefaultCredentialsError as e:
    print(f"Error with credentials: {str(e)}", file=sys.stderr)
    sys.exit(1)

# Construct the parent path for listing Docker images
parent = f"projects/{project}/locations/{location}/repositories/{repo}"
docker_images = list_docker_images(client, parent)

# Identify old images based on update time and tags
old_pr_image_tags = [
    img
    for img in docker_images
    if img.update_time.timestamp_pb().ToDatetime() < date and any("commit" in tag for tag in img.tags)
]
old_untagged_images = [
    img for img in docker_images if img.update_time.timestamp_pb().ToDatetime() < date and len(img.tags) == 0
]
old_image_tags = old_pr_image_tags + old_untagged_images

# Filter and delete old images
for image in images:
    old_images = [img for img in old_image_tags if img.uri.startswith(f"{registry}/{image}@")]

    for img in old_images:
        print(f"Deleting image {img.uri}")
        delete_docker_image(client, img.name, dry_run)
