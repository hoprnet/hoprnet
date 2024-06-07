{ nixpkgs
, rust-overlay
, crane
, foundry
, solc
, localSystem
, crossSystem ? localSystem
}:
let
  pkgs = import nixpkgs {
    inherit crossSystem localSystem;
    overlays = [ rust-overlay.overlays.default foundry.overlay solc.overlay ];
  };

  solcDefault = solc.mkDefault pkgs pkgs.pkgsBuildHost.solc_0_8_19;

  lib = pkgs.pkgsBuildHost.lib;

  envCase = triple: lib.strings.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] triple);

  # `hostPlatform` is the cross-compilation output platform;
  # `buildPlatform` is the platform we are compiling on
  buildPlatform = pkgs.stdenv.buildPlatform;
  hostPlatform = pkgs.stdenv.hostPlatform;

  rustBin = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
  rustToolchain = rustBin.override { targets = [ hostPlatform.config ]; };
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  buildEnv = {
    CARGO_BUILD_TARGET = hostPlatform.config;
    "CARGO_TARGET_${envCase hostPlatform.config}_LINKER" = "${pkgs.stdenv.cc.targetPrefix}cc";
    HOST_CC = "${pkgs.stdenv.cc.nativePrefix}cc";
  };
in
{
  inherit rustToolchain;

  callPackage = (package: args:
    let crate = pkgs.callPackage package (args // { inherit solcDefault craneLib; });
    in
    # Override the derivation to add cross-compilation environment variables.
    crate.overrideAttrs (previous: buildEnv // {
      # We also have to override the `cargoArtifacts` derivation with the same changes.
      cargoArtifacts = previous.cargoArtifacts.overrideAttrs (previous: buildEnv);
    }));
}
