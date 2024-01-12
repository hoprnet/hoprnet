{
  description = "hoprnet monorepo";


  inputs.flake-utils.url = github:numtide/flake-utils;
  inputs.flake-parts.url = github:hercules-ci/flake-parts;
  inputs.nixpkgs.url = github:NixOS/nixpkgs/master;
  # using bugfix for macos libcurl:
  # https://github.com/oxalica/rust-overlay/pull/149
  inputs.rust-overlay.url = github:oxalica/rust-overlay/647bff9f5e10d7f1756d86eee09831e6b1b06430;
  inputs.crane.url = github:ipetkov/crane;
  inputs.foundry.url = github:shazow/foundry.nix/monthly;
  inputs.solc.url = github:hellwolf/solc.nix;

  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.foundry.inputs.nixpkgs.follows = "nixpkgs";
  inputs.foundry.inputs.flake-utils.follows = "flake-utils";
  inputs.solc.inputs.nixpkgs.follows = "nixpkgs";
  inputs.solc.inputs.flake-utils.follows = "flake-utils";

  outputs = { self, nixpkgs, flake-parts, rust-overlay, crane, foundry, solc, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          fs = lib.fileset;
          overlays = [ (import rust-overlay) foundry.overlay solc.overlay ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          crateNameFromCargoToml = craneLib.crateNameFromCargoToml {
            cargoToml = ./packages/hoprd/crates/hoprd-hoprd/Cargo.toml;
          };
          solcDefault = with pkgs; (solc.mkDefault pkgs solc_0_8_19);
          ethereumBindings = pkgs.stdenv.mkDerivation {
            pname = "${crateNameFromCargoToml.pname}-ethereum-bindings";
            version = crateNameFromCargoToml.version;
            src = lib.fileset.toSource {
              root = ./.;
              fileset = fs.unions [
                (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
                (fs.fileFilter (file: file.hasExt "sol") ./packages/ethereum/contracts/src)
                ./packages/ethereum/contracts/foundry.toml
                ./packages/ethereum/contracts/remappings.txt
              ];
            };
            buildInputs = with pkgs; [ foundry-bin solcDefault ];
            buildPhase = ''
              cd ./packages/ethereum/contracts
              sed -i "s|solc = .*|solc = \"${solcDefault}/bin/solc\"|g" foundry.toml
              forge bind --bindings-path ../crates/bindings --crate-name bindings \
                --overwrite --select '^Hopr.*?(Boost|[^t])$$'
            '';
            installPhase = ''
              echo `pwd`
              cp -r --no-preserve=mode,ownership ../crates/bindings $out/
            '';
          };
          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor/cargo
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              ./packages/hoprd/crates/hopr-lib/data
            ];
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];
          buildInputs = with pkgs; [
            foundry-bin
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
            with darwin.apple_sdk.frameworks; [
              SystemConfiguration
            ]
          );
          commonArgs = {
            inherit (crateNameFromCargoToml) pname version;
            inherit src buildInputs nativeBuildInputs;
            CARGO_HOME = ".cargo";
            cargoVendorDir = "./vendor/cargo";
            # disable running tests automatically for now
            doCheck = false;
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            extraDummyScript = ''
              rm -rf $out/vendor/cargo
              cp -r --no-preserve=mode,ownership ${src}/vendor/cargo $out/vendor/
            '';
          });
          hoprd = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            preBuild = ''
              cp -r ${ethereumBindings}/src ./packages/ethereum/crates/bindings/
              cp .cargo/config.toml vendor/cargo/
            '';
          });
          # FIXME: the docker image built is not working on macOS arm platforms
          # and will simply be a no-op. Likely, some form of cross-compilation
          # or distributed build is required.
          hoprdDocker = pkgs.dockerTools.buildImage {
            name = "hoprd";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            copyToRoot = [ hoprd ];
            config = {
              Cmd = [
                "${hoprd}/bin/hoprd"
              ];
            };
          };
        in
        {
          packages = {
            inherit hoprd hoprdDocker;
            default = hoprd;
          };
          devShells.default = import ./shell.nix {
            inherit pkgs;
            inputsFrom = [ hoprd ];
          };
        };
      systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin" ];
      flake = {
        overlays = [
          rust-overlay.overlays
        ];
      };
    };
}
