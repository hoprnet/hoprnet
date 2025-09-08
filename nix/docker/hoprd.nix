# hoprd.nix - HOPRD Docker image definitions
#
# Defines Docker images for the HOPRD daemon with different profiles.
# Images are built as layered containers for efficient caching and smaller sizes.

{
  pkgs,
  dockerBuilder,
  packages,
}:

let
  # Create the Docker entrypoint script
  # Handles container-specific initialization and configuration
  mkDockerEntrypoint = pkgs.writeShellScriptBin "docker-entrypoint.sh" ''
    set -euo pipefail

    # If the default listen host has not been set by the user,
    # we will set it to the container's IP address.
    # This ensures the node is accessible within Docker networks.
    # Defaults to a random port (0) if not specified.

    listen_host="''${HOPRD_DEFAULT_SESSION_LISTEN_HOST:-}"
    listen_host_preset_ip="''${listen_host%%:*}"
    listen_host_preset_port="''${listen_host#*:}"

    if [ -z "''${listen_host_preset_ip:-}" ]; then
      listen_host_ip="$(hostname -i)"

      if [ -z "''${listen_host_preset_port:-}" ]; then
        listen_host="''${listen_host_ip}:0"
      else
        listen_host="''${listen_host_ip}:''${listen_host_preset_port}"
      fi
    fi

    export HOPRD_DEFAULT_SESSION_LISTEN_HOST="''${listen_host}"

    # Allow execution of auxiliary commands if provided
    # Otherwise default to running hoprd
    if [ -n "''${1:-}" ] && [ -f "/bin/''${1:-}" ] && [ -x "/bin/''${1:-}" ]; then
      # Execute specified command if it exists and is executable
      exec "$@"
    else
      # Default to running hoprd with any provided arguments
      exec /bin/hoprd "$@"
    fi
  '';

  # Profile-specific dependencies for debugging and profiling
  profileDeps = with pkgs; [
    gdb # GNU debugger for debugging
    rust-bin.stable.latest.minimal # Minimal Rust toolchain for analysis
    valgrind # Memory debugging and profiling
    gnutar # For extracting pcap files from container
  ];

  # Base Docker image configuration for HOPRD
  mkHoprdDocker =
    {
      package,
      extraDeps ? [ ],
      nameSuffix ? "",
    }:
    dockerBuilder {
      inherit pkgs;
      name = "hoprd${nameSuffix}";
      extraContents = [
        (mkDockerEntrypoint)
        package
      ]
      ++ extraDeps;
      Entrypoint = [ "/bin/docker-entrypoint.sh" ];
      Cmd = [ "hoprd" ];
    };
in
{
  # Production Docker image - minimal size, optimized build
  hoprd-docker = mkHoprdDocker {
    package = packages.hoprd-x86_64-linux;
  };

  # Development Docker image - dev profile for debugging
  hoprd-dev-docker = mkHoprdDocker {
    package = packages.hoprd-x86_64-linux-dev;
    nameSuffix = "-dev";
  };

  # Profiling Docker image - includes debugging tools
  hoprd-profile-docker = mkHoprdDocker {
    package = packages.hoprd-x86_64-linux-profile;
    extraDeps = profileDeps;
    nameSuffix = "-profile";
  };
}
