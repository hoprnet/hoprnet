#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["google-cloud-artifact-registry==1.15.2","google-auth==2.38.0"]
# ///

from datetime import UTC, datetime, timedelta
from google.auth.exceptions import DefaultCredentialsError
from google.cloud import artifactregistry_v1
import argparse
import asyncio
import itertools
import re
import subprocess
import sys


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
        print(f"Listing Docker images in {parent}")
        request = artifactregistry_v1.ListDockerImagesRequest(parent=parent)
        return [image for image in client.list_docker_images(request=request)]
    except Exception as e:
        print(f"Error listing Docker images: {str(e)}", file=sys.stderr)
        sys.exit(1)

async def delete_docker_images_list(images):
    asyncio.gather(
        *[
            asyncio.wait_for(
                delete_docker_image(img.uri, dry_run),
                timeout=60
            )
            for img in images
        ]
    )

def delete_docker_image(uri, dry_run):
    """
    Deletes a Docker image by its uri.

    Args:
        uri: The uri of the Docker image to delete.
        dry_run: If True, simulates the deletion without making changes.

    Raises:
        SystemExit: If an error occurs while deleting the Docker image.
    """
    if dry_run:
        print(f"Dry-run mode: Would delete image {uri}")
        return
    try:
        print(f"Deleting Docker image {uri}")
        # need to use gcloud cli because docker image deletion is not
        # supported by the Artifact Registry client
        cmd = f"gcloud artifacts docker images delete {uri} --async --delete-tags -q"
        subprocess.run(cmd.split(), check=True)
    except Exception as e:
        print(f"Error deleting Docker image: {str(e)}", file=sys.stderr)
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
date = datetime.now(UTC) - timedelta(days=days)
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

async def main():
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
        if img.update_time.timestamp_pb().ToDatetime().astimezone(UTC) < date and any("commit" in tag for tag in img.tags)
    ]
    old_untagged_images = [
        img for img in docker_images if img.update_time.timestamp_pb().ToDatetime().astimezone(UTC) < date and len(img.tags) == 0
    ]
    old_image_tags = old_pr_image_tags + old_untagged_images
    
    # Filter and delete old images
    for image in images:
        old_images = [img for img in old_image_tags if img.uri.startswith(f"{registry}/{image}@")]
    
        for old_images_part in itertools.batched(old_images, 20):
            await delete_docker_images_list(old_images_part)
    
asyncio.run(main())
