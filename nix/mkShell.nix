{}:

{ config
, pkgs
, crane
, shellHook ? ""
, shellPackages ? [ ]
, useRustNightly ? false
}:
let
  cargoTarget = pkgs.stdenv.buildPlatform.config;
  rustToolchain =
    if useRustNightly
    then pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)
    else
      (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
        ../rust-toolchain.toml).override { targets = [ cargoTarget ]; };
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  minimumPackages = with pkgs; [
    bash
    coreutils
    curl
    findutils
    gnumake
    jq
    lsof
    mold
    openssl
    patchelf
    pkg-config
    time
    which
    yq-go

    ## formatting
    config.treefmt.build.wrapper
  ] ++
  (lib.attrValues config.treefmt.build.programs) ++
  lib.optionals stdenv.isLinux [ autoPatchelfHook ];
  packages = minimumPackages ++ shellPackages;
in
craneLib.devShell {
  inherit shellHook packages;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ([ pkgs.pkgsBuildHost.openssl pkgs.pkgsBuildHost.curl ] ++
    pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.pkgsBuildHost.libgcc.lib ]);
}
