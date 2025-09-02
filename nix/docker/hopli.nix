# hopli.nix - HOPLI Docker image definitions
#
# Defines Docker images for the HOPLI CLI tool with different profiles.
# Images include necessary environment variables for contract interaction.

{ pkgs
, dockerBuilder
, packages
}:

let
  # Profile-specific dependencies for debugging
  profileDeps = with pkgs; [
    gdb              # GNU debugger
    rust-bin.stable.latest.minimal  # Minimal Rust toolchain
    valgrind         # Memory debugging
    gnutar           # Archive extraction
  ];

  # Base Docker image configuration for HOPLI
  mkHopliDocker = { package, extraDeps ? [], nameSuffix ? "" }:
    dockerBuilder {
      inherit pkgs;
      name = "hopli${nameSuffix}";
      extraContents = [ package ] ++ extraDeps;
      Entrypoint = [ "/bin/hopli" ];
      # Set environment variables for contract interaction
      env = [
        "ETHERSCAN_API_KEY=placeholder"  # Default placeholder, override at runtime
        "HOPLI_CONTRACTS_ROOT=${package}/ethereum/contracts"  # Path to contract data
      ];
    };
in
{
  # Production Docker image
  hopli-docker = mkHopliDocker {
    package = packages.hopli-x86_64-linux;
  };

  # Development Docker image
  hopli-dev-docker = mkHopliDocker {
    package = packages.hopli-x86_64-linux-dev;
    nameSuffix = "-dev";
  };

  # Profiling Docker image with debugging tools
  hopli-profile-docker = mkHopliDocker {
    package = packages.hopli-x86_64-linux;  # Note: Uses regular build, not dev
    extraDeps = profileDeps;
    nameSuffix = "-profile";
  };
}