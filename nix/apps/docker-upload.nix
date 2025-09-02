# docker-upload.nix - Docker image upload utilities
#
# Provides functions to create upload scripts for Docker images.
# These scripts build images with Nix and upload them to container registries.

{
  pkgs,
  flake-utils,
}:

rec {
  # Create a script that builds and uploads a Docker image to a registry
  # Uses Google Cloud Registry authentication via GOOGLE_ACCESS_TOKEN
  mkDockerUploadScript =
    image:
    pkgs.writeShellScriptBin "docker-image-upload" ''
      set -eu

      # Build the Docker image using Nix
      # --no-link prevents creating a result symlink
      # --print-out-paths returns the store path of the built image
      OCI_ARCHIVE="$(nix build --no-link --print-out-paths ${image})"

      # Upload the image to the registry using skopeo
      # Requires GOOGLE_ACCESS_TOKEN and IMAGE_TARGET environment variables
      ${pkgs.skopeo}/bin/skopeo copy --insecure-policy \
        --dest-registry-token="$GOOGLE_ACCESS_TOKEN" \
        "docker-archive:$OCI_ARCHIVE" "docker://$IMAGE_TARGET"
    '';

  # Create a flake app from a Docker upload script
  # This makes the script runnable via `nix run`
  mkDockerUploadApp =
    image:
    flake-utils.lib.mkApp {
      drv = mkDockerUploadScript image;
    };
}
