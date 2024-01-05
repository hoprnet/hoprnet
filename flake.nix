{
  description = "hoprnet monorepo";

  inputs.flake-parts.url = github:hercules-ci/flake-parts;
  inputs.nixpkgs.url = github:NixOS/nixpkgs/master;
  # using bugfix for macos libcurl:
  # https://github.com/oxalica/rust-overlay/pull/149
  inputs.rust-overlay.url = github:oxalica/rust-overlay/647bff9f5e10d7f1756d86eee09831e6b1b06430;
  #inputs.crane.url = github:ipetkov/crane;
  inputs.crane.url = "git+file:///Users/tbr/work/hopr/crane";

  inputs.rust-overlay.inputs = {
    nixpkgs.follows = "nixpkgs";
  };

  inputs.crane.inputs = {
    nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-parts, rust-overlay, crane, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          #vendorFilter = path: _type: builtins.match "^vendor/cargo" path != null;
          #vendorOrCargo = path: type: (vendorFilter path type) || (craneLib.filterCargoSources path type);
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.gitTracked ./.;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];
          buildInputs = with pkgs; [
          ];
          crateNameFromCargoToml = craneLib.crateNameFromCargoToml {
            cargoToml = ./packages/hoprd/crates/hoprd-hoprd/Cargo.toml;
          };
          commonArgs = {
            inherit (crateNameFromCargoToml) pname version;
            inherit src buildInputs nativeBuildInputs;
            cargoVendorDir = "./vendor/cargo";
            cargoExtraArgs = "--config .cargo/config.toml";
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            extraDummyScript = ''
              rm -rf $out/vendor/cargo
              cp -r --no-preserve=mode,ownership ${src}/vendor/cargo $out/vendor/
              cp ${src}/.cargo/config.toml $out/vendor/cargo/
            '';
          });
          bin = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          packages = {
            inherit bin;
            default = bin;
          };
          devShells.default = import ./shell.nix {
            inherit pkgs;
            inputsFrom = [ bin ];
          };
        };
      systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin"];
      flake = {
        overlays = [
          rust-overlay.overlays
        ];
      };
    };
}
