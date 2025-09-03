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
  #
  # Required environment variables:
  #   - GOOGLE_ACCESS_TOKEN: Access token for Google Cloud Registry authentication
  #   - IMAGE_TARGET: Full registry path for the target image (e.g., gcr.io/project/image:tag)
  #
  # Optional environment variables:
  #   - SKOPEO_INSECURE_POLICY=1: Enable insecure policy mode (bypasses signature verification)
  mkDockerUploadScript =
    image:
    pkgs.writeShellScriptBin "docker-image-upload" ''
      set -euo pipefail

      # Validation function for environment variables
      validate_env_var() {
        local var_name="$1"
        local var_value="''${!var_name:-}"
        
        if [[ -z "$var_value" ]]; then
          echo "ERROR: Required environment variable $var_name is not set or empty" >&2
          echo "Usage: Set $var_name before running this script" >&2
          exit 1
        fi
      }

      # Validate required environment variables
      validate_env_var "GOOGLE_ACCESS_TOKEN"
      validate_env_var "IMAGE_TARGET"

      # Build the Docker image using Nix
      # --no-link prevents creating a result symlink
      # --print-out-paths returns the store path of the built image
      if ! OCI_ARCHIVE="$(nix build --no-link --print-out-paths ${image} 2>/dev/null)"; then
        echo "ERROR: Failed to build Docker image with Nix" >&2
        exit 2
      fi

      # Validate build output
      if [[ -z "$OCI_ARCHIVE" ]]; then
        echo "ERROR: Nix build returned empty output path" >&2
        exit 2
      fi

      if [[ ! -f "$OCI_ARCHIVE" ]]; then
        echo "ERROR: Built image archive does not exist: $OCI_ARCHIVE" >&2
        exit 2
      fi

      echo "Docker image built successfully: $OCI_ARCHIVE"

      # Prepare skopeo command with security options
      skopeo_args=(
        "copy"
        "--dest-registry-token=$GOOGLE_ACCESS_TOKEN"
      )

      # Add insecure policy flag only if explicitly requested
      if [[ "''${SKOPEO_INSECURE_POLICY:-}" == "1" ]]; then
        echo "WARNING: Using insecure policy mode (signature verification disabled)" >&2
        skopeo_args+=("--insecure-policy")
      fi

      skopeo_args+=(
        "docker-archive:$OCI_ARCHIVE"
        "docker://$IMAGE_TARGET"
      )

      # Upload the image to the registry using skopeo
      echo "Uploading image to registry: $IMAGE_TARGET"
      if ! ${pkgs.skopeo}/bin/skopeo "''${skopeo_args[@]}"; then
        echo "ERROR: Failed to upload image to registry" >&2
        exit 3
      fi

      echo "Image uploaded successfully to: $IMAGE_TARGET"
    '';

  # Create a flake app from a Docker upload script
  # This makes the script runnable via `nix run`
  mkDockerUploadApp =
    image:
    flake-utils.lib.mkApp {
      drv = mkDockerUploadScript image;
    };
}
