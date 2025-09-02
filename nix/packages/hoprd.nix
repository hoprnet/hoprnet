# hoprd.nix - HOPRD daemon package definitions
#
# Defines all variants of the HOPRD daemon for different platforms and profiles.
# HOPRD is the main HOPR node software that participates in the HOPR network.

{ lib
, builders
, sources
, hoprdCrateInfo
, rev
, buildPlatform
}:

let
  # Common build arguments for all HOPRD variants
  # These are shared across all build configurations
  mkHoprdBuildArgs = { src, depsSrc }:
    {
      inherit src depsSrc rev;
      cargoExtraArgs = "-p hoprd-api";  # Build the hoprd-api package specifically
      cargoToml = ./../../hoprd/hoprd/Cargo.toml;
    };
in
{
  # Development builds - for local testing and debugging
  hoprd = builders.local.callPackage ../rust-package.nix (
    mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hoprd-dev = builders.local.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // {
      CARGO_PROFILE = "dev";
      cargoExtraArgs = "-F capture";  # Enable profiling capture
    }
  );

  # Production builds - optimized for deployment
  # x86_64 Linux builds with static linking for maximum portability
  hoprd-x86_64-linux = builders.x86_64-linux.callPackage ../rust-package.nix (
    mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hoprd-x86_64-linux-profile = builders.x86_64-linux.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      cargoExtraArgs = "-F capture";  # Enable profiling capture
    }
  );

  hoprd-x86_64-linux-dev = builders.x86_64-linux.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // {
      CARGO_PROFILE = "dev";
      cargoExtraArgs = "-F capture";
    }
  );

  # ARM64 Linux builds for ARM servers and devices
  hoprd-aarch64-linux = builders.aarch64-linux.callPackage ../rust-package.nix (
    mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hoprd-aarch64-linux-profile = builders.aarch64-linux.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      cargoExtraArgs = "-F capture";
    }
  );

  # macOS builds - require building from Darwin systems
  # x86_64 macOS (Intel Macs)
  hoprd-x86_64-darwin = builders.x86_64-darwin.callPackage ../rust-package.nix (
    mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hoprd-x86_64-darwin-profile = builders.x86_64-darwin.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      cargoExtraArgs = "-F capture";
    }
  );

  # ARM64 macOS (Apple Silicon)
  hoprd-aarch64-darwin = builders.aarch64-darwin.callPackage ../rust-package.nix (
    mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }
  );

  hoprd-aarch64-darwin-profile = builders.aarch64-darwin.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      cargoExtraArgs = "-F capture";
    }
  );

  # Test and quality assurance builds
  hoprd-test = builders.local.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.test; depsSrc = sources.deps; }) // {
      runTests = true;
    }
  );

  hoprd-test-nightly = builders.localNightly.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.test; depsSrc = sources.deps; }) // {
      runTests = true;
      cargoExtraArgs = "-Z panic-abort-tests";  # Nightly feature for test optimization
    }
  );

  hoprd-clippy = builders.local.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      runClippy = true;  # Run Clippy linter
    }
  );

  hoprd-bench = builders.local.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      runBench = true;  # Run benchmarks
    }
  );

  # Candidate build - used for smoke testing before release
  # Builds as static binary on Linux x86_64 for better test coverage
  hoprd-candidate =
    if buildPlatform.isLinux && buildPlatform.isx86_64 then
      builders.x86_64-linux.callPackage ../rust-package.nix (
        (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
          CARGO_PROFILE = "candidate";
        }
      )
    else
      builders.local.callPackage ../rust-package.nix (
        (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
          CARGO_PROFILE = "candidate";
        }
      );

  # Documentation build using nightly Rust for unstable doc features
  hoprd-docs = builders.localNightly.callPackage ../rust-package.nix (
    (mkHoprdBuildArgs { src = sources.main; depsSrc = sources.deps; }) // { 
      buildDocs = true;
    }
  );
}