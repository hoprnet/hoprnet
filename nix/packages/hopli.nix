# hopli.nix - HOPLI CLI tool package definitions
#
# Defines all variants of the HOPLI command-line interface tool.
# HOPLI is used for interacting with HOPR smart contracts and managing nodes.

{ lib
, builders
, sources
, rev
, buildPlatform
}:

let
  # Common build arguments for all HOPLI variants
  # Includes post-install step to copy contract addresses
  mkHopliBuildArgs = { src, depsSrc }:
    {
      inherit src depsSrc rev;
      cargoToml = ./../../hopli/Cargo.toml;
      # Copy contract addresses to output for runtime access
      postInstall = ''
        mkdir -p $out/ethereum/contracts
        cp ethereum/contracts/contracts-addresses.json $out/ethereum/contracts/
      '';
    };
in
{
  # Development builds
  hopli = builders.local.callPackage ../rust-package.nix (
    mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hopli-dev = builders.local.callPackage ../rust-package.nix (
    (mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      CARGO_PROFILE = "dev";
    }
  );

  # Production builds - x86_64 Linux with static linking
  hopli-x86_64-linux = builders.x86_64-linux.callPackage ../rust-package.nix (
    mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hopli-x86_64-linux-dev = builders.x86_64-linux.callPackage ../rust-package.nix (
    (mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      CARGO_PROFILE = "dev";
    }
  );

  # ARM64 Linux builds
  hopli-aarch64-linux = builders.aarch64-linux.callPackage ../rust-package.nix (
    mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  # macOS builds - require building from Darwin systems
  # x86_64 macOS (Intel Macs)
  hopli-x86_64-darwin = builders.x86_64-darwin.callPackage ../rust-package.nix (
    mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  # ARM64 macOS (Apple Silicon)
  hopli-aarch64-darwin = builders.aarch64-darwin.callPackage ../rust-package.nix (
    mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  # Quality assurance builds
  hopli-clippy = builders.local.callPackage ../rust-package.nix (
    (mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      runClippy = true;  # Run Clippy linter
    }
  );

  # Candidate build for smoke testing
  # Builds as static binary on Linux x86_64 for better test coverage
  hopli-candidate =
    if buildPlatform.isLinux && buildPlatform.isx86_64 then
      builders.x86_64-linux.callPackage ../rust-package.nix (
        (mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
          CARGO_PROFILE = "candidate";
        }
      )
    else
      builders.local.callPackage ../rust-package.nix (
        (mkHopliBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
          CARGO_PROFILE = "candidate";
        }
      );
}