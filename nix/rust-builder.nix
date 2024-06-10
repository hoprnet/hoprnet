{ crane
, crossSystem ? localSystem
, foundry
, localSystem
, nixpkgs
, rust-overlay
, solc
, useRustNightly ? false
}:
let
  pkgs = import nixpkgs {
    inherit crossSystem localSystem;
    overlays = [ rust-overlay.overlays.default solc.overlay ];
  };

  solcDefault = solc.mkDefault pkgs pkgs.pkgsBuildHost.solc_0_8_19;

  # the foundry overlay uses the hostPlatform, so we need to use a
  # localSystem-only pkgs to get the correct architecture
  pkgsLocal = import nixpkgs {
    system = localSystem;
    overlays = [ foundry.overlay ];
  };
  foundryBin = pkgsLocal.foundry-bin;

  lib = pkgs.pkgsBuildHost.lib;

  envCase = triple: lib.strings.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] triple);

  # `hostPlatform` is the cross-compilation output platform;
  # `buildPlatform` is the platform we are compiling on
  buildPlatform = pkgs.stdenv.buildPlatform;
  hostPlatform = pkgs.stdenv.hostPlatform;

  cargoTarget =
    if hostPlatform.config == "armv7l-unknown-linux-gnueabihf" then
      "armv7-unknown-linux-gnueabihf" else hostPlatform.config;

  rustToolchain =
    if useRustNightly
    then pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)
    else
      (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
        ../rust-toolchain.toml).override { targets = [ cargoTarget ]; };

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  buildEnv = {
    CARGO_BUILD_TARGET = cargoTarget;
    "CARGO_TARGET_${envCase cargoTarget}_LINKER" = "${pkgs.stdenv.cc.targetPrefix}cc";
    HOST_CC = "${pkgs.stdenv.cc.nativePrefix}cc";
  };
in
{
  inherit rustToolchain;

  callPackage = (package: args:
    let crate = pkgs.callPackage package (args // { inherit foundryBin solcDefault craneLib; });
    in
    # Override the derivation to add cross-compilation environment variables.
    crate.overrideAttrs (previous: buildEnv // {
      # We also have to override the `cargoArtifacts` derivation with the same changes.
      cargoArtifacts = previous.cargoArtifacts.overrideAttrs (previous: buildEnv);
    }));
}
