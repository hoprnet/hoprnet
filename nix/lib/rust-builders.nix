# rust-builders.nix - Rust cross-compilation builder factory
#
# Provides functions to create Rust builders for different target platforms.
# Supports cross-compilation, static linking, and nightly toolchains.

{
  nixpkgs,
  rust-overlay,
  crane,
  foundry,
  solc,
  localSystem,
}:

rec {
  # Create a Rust builder for the local platform
  # This is the default builder used for development
  mkLocalBuilder =
    {
      useRustNightly ? false,
    }:
    import ../rust-builder.nix {
      inherit
        nixpkgs
        rust-overlay
        crane
        foundry
        solc
        localSystem
        useRustNightly
        ;
    };

  # Create a Rust builder for x86_64 Linux with musl (static linking)
  # Used for production Linux deployments
  mkX86_64LinuxBuilder =
    { }:
    import ../rust-builder.nix {
      inherit
        nixpkgs
        rust-overlay
        crane
        foundry
        solc
        localSystem
        ;
      crossSystem = (import nixpkgs { inherit localSystem; }).lib.systems.examples.musl64;
      isCross = true;
      isStatic = true;
    };

  # Create a Rust builder for aarch64 Linux with musl (static linking)
  # Used for ARM64 Linux deployments (e.g., AWS Graviton)
  mkAarch64LinuxBuilder =
    { }:
    import ../rust-builder.nix {
      inherit
        nixpkgs
        rust-overlay
        crane
        foundry
        solc
        localSystem
        ;
      crossSystem =
        (import nixpkgs { inherit localSystem; }).lib.systems.examples.aarch64-multiplatform-musl;
      isCross = true;
      isStatic = true;
    };

  # Create a Rust builder for x86_64 macOS
  # Note: Must be built from a Darwin system for proper code signing
  mkX86_64DarwinBuilder =
    { }:
    import ../rust-builder.nix {
      inherit
        nixpkgs
        rust-overlay
        crane
        foundry
        solc
        localSystem
        ;
      crossSystem = (import nixpkgs { inherit localSystem; }).lib.systems.examples.x86_64-darwin;
      isCross = true;
    };

  # Create a Rust builder for aarch64 macOS (Apple Silicon)
  # Note: Must be built from a Darwin system for proper code signing
  mkAarch64DarwinBuilder =
    { }:
    import ../rust-builder.nix {
      inherit
        nixpkgs
        rust-overlay
        crane
        foundry
        solc
        localSystem
        ;
      crossSystem = (import nixpkgs { inherit localSystem; }).lib.systems.examples.aarch64-darwin;
      isCross = true;
    };

  # Helper function to create all platform builders at once
  # Returns an attribute set with all available builders
  mkAllBuilders =
    { }:
    {
      local = mkLocalBuilder { };
      localNightly = mkLocalBuilder { useRustNightly = true; };
      x86_64-linux = mkX86_64LinuxBuilder { };
      aarch64-linux = mkAarch64LinuxBuilder { };
      x86_64-darwin = mkX86_64DarwinBuilder { };
      aarch64-darwin = mkAarch64DarwinBuilder { };
    };
}
