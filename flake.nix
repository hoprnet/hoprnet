{
  description = "hoprnet monorepo";


  inputs.flake-utils.url = github:numtide/flake-utils;
  inputs.flake-parts.url = github:hercules-ci/flake-parts;
  inputs.nixpkgs.url = github:NixOS/nixpkgs/master;
  # using bugfix for macos libcurl:
  # https://github.com/oxalica/rust-overlay/pull/149
  inputs.rust-overlay.url = github:oxalica/rust-overlay/647bff9f5e10d7f1756d86eee09831e6b1b06430;
  # using a fork with an added source filter
  inputs.crane.url = github:hoprnet/crane/tb/20240117-find-filter;
  inputs.foundry.url = github:shazow/foundry.nix/monthly;
  inputs.solc.url = github:hellwolf/solc.nix;

  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.foundry.inputs.nixpkgs.follows = "nixpkgs";
  inputs.foundry.inputs.flake-utils.follows = "flake-utils";
  inputs.solc.inputs.nixpkgs.follows = "nixpkgs";
  inputs.solc.inputs.flake-utils.follows = "flake-utils";

  outputs = { self, nixpkgs, flake-utils, flake-parts, rust-overlay, crane, foundry, solc, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          fs = lib.fileset;
          overlays = [ (import rust-overlay) foundry.overlay solc.overlay ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          craneLibNightly = (crane.mkLib pkgs).overrideToolchain rustNightly;
          hoprdCrateInfoOriginal = craneLib.crateNameFromCargoToml {
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          hoprdCrateInfo = {
            pname = "hoprd";
            # normalize the version to not include any suffixes so the cache
            # does not get busted
            version = pkgs.lib.strings.concatStringsSep "."
              (pkgs.lib.lists.take 3 (builtins.splitVersion hoprdCrateInfoOriginal.version));
          };
          solcDefault = with pkgs; (solc.mkDefault pkgs solc_0_8_19);
          depsSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor/cargo
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.name == "Cargo.toml") ./.)
            ];
          };
          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./vendor/cargo
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              ./hopr/hopr-lib/data
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/remappings.txt
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
            ];
          };
          nativeBuildInputs = with pkgs; [
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
            inherit buildInputs nativeBuildInputs;
            CARGO_HOME = ".cargo";
            cargoVendorDir = "vendor/cargo";
            # disable running tests automatically for now
            doCheck = false;
          };
          hopliCrateInfo = craneLib.crateNameFromCargoToml {
            cargoToml = ./hopli/Cargo.toml;
          };
          rustPackageDeps = { pname, version, builder ? craneLib.buildDepsOnly}: builder (commonArgs // {
            inherit pname version;
            src = depsSrc;
            cargoExtraArgs = "--offline -p ${pname}";
            extraDummyScript = ''
              mkdir -p $out/vendor/cargo
              cp -r --no-preserve=mode,ownership ${src}/vendor/cargo $out/vendor/
              echo "# placeholder" > $out/vendor/cargo/config.toml
            '';
          });
          rustPackage = { pname, version, cargoArtifacts}: craneLib.buildPackage (commonArgs // {
            inherit pname version cargoArtifacts src;
            cargoExtraArgs = "--offline -p ${pname}";
            preConfigure = ''
              echo "# placeholder" > vendor/cargo/config.toml
              sed -i "s|solc = .*|solc = \"${solcDefault}/bin/solc\"|g" ethereum/contracts/foundry.toml
            '';
          });
          rustPackageTest = { pname, version, cargoArtifacts}: craneLib.cargoTest (commonArgs // {
            inherit pname version cargoArtifacts src;
            cargoExtraArgs = "--offline -p ${pname}";
            preConfigure = ''
              echo "# placeholder" > vendor/cargo/config.toml
              sed -i "s|solc = .*|solc = \"${solcDefault}/bin/solc\"|g" ethereum/contracts/foundry.toml
            '';
            doCheck = true;
          });
          hoprd = rustPackage (hoprdCrateInfo // {cargoArtifacts = rustPackageDeps hoprdCrateInfo;});
          hopli = rustPackage (hopliCrateInfo // {cargoArtifacts = rustPackageDeps hopliCrateInfo;});
          hoprd-test = rustPackageTest (hoprdCrateInfo // {cargoArtifacts = rustPackageDeps hoprdCrateInfo;});
          hopli-test = rustPackageTest (hopliCrateInfo // {cargoArtifacts = rustPackageDeps hopliCrateInfo;});
          # FIXME: the docker image built is not working on macOS arm platforms
          # and will simply lead to a non-working image. Likely, some form of
          # cross-compilation or distributed build is required.
          hoprd-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hoprd";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = [ hoprd ];
            config = {
              Entrypoint = [
                "/bin/hoprd"
              ];
            };
          };
          hopli-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hopli";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = [ hopli ];
            config = {
              Entrypoint = [
                "/bin/hopli"
              ];
            };
          };
          anvilSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              ./ethereum/contracts
              ./scripts/run-local-anvil.sh
            ];
          };
          anvil-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hopr-anvil";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = [ pkgs.foundry-bin anvilSrc pkgs.tini pkgs.runtimeShellPackage ];
            enableFakechroot = true;
            fakeRootCommands = ''
              #!${pkgs.runtimeShell}
              /scripts/run-local-anvil.sh
              sleep 2
              lsof -i :8545 -s TCP:LISTEN -t | xargs -I {} -n 1 kill {} || :
              rm -rf /ethereum/contracts/broadcast/
              rm -f /tmp/*.log
              rm -f /.anvil.state.json
            '';
            config = {
              Cmd = [
                "/bin/tini"
                "--"
                "/scripts/run-local-anvil.sh"
                "-s"
                "-f"
              ];
            };
          };
          dockerImageUploadScript = image: pkgs.writeShellScriptBin "docker-image-upload" ''
            set -eu
            OCI_ARCHIVE="$(nix build --no-link --print-out-paths ${image})"
            ${pkgs.skopeo}/bin/skopeo copy --insecure-policy \
              --dest-registry-token="$GOOGLE_ACCESS_TOKEN" \
              "docker-archive:$OCI_ARCHIVE" "docker://$IMAGE_TARGET"
          '';
          hoprd-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hoprd-docker;
          };
          hopli-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hopli-docker;
          };
          docs = craneLibNightly.cargoDoc (commonArgs // {
            inherit src;
            pname = "hopr";
            version = hoprdCrateInfo.version;
            cargoArtifacts = rustPackageDeps (hoprdCrateInfo // { builder = craneLibNightly.buildDepsOnly;});
            buildPhaseCargoCommand = "cargo doc --offline --no-deps";
            preConfigure = ''
              echo "# placeholder" > vendor/cargo/config.toml
              sed -i "s|solc = .*|solc = \"${solcDefault}/bin/solc\"|g" ethereum/contracts/foundry.toml
            '';
            postBuild = ''
              ${pkgs.pandoc}/bin/pandoc -f markdown+hard_line_breaks -t html README.md > readme.html
              ${pkgs.html-tidy}/bin/tidy -q -i target/doc/index.html > index.html || :
              sed '/<section id="main-content" class="content">/ r readme.html' index.html > target/doc/index.html
              rm readme.html index.html
            '';
          });
          smoke-tests = pkgs.stdenv.mkDerivation {
            pname = "hoprd-smoke-tests";
            version = hoprdCrateInfo.version;
            src = fs.toSource {
              root = ./.;
              fileset = fs.unions [
                (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
                ./tests
                ./scripts
                ./ethereum/contracts/foundry.toml
                ./ethereum/contracts/remappings.txt
              ];
            };
            buildInputs = with pkgs; [
              foundry-bin
              solcDefault
              hopli
              hoprd
              python39
            ];
            buildPhase = ''
              unset SOURCE_DATE_EPOCH
              python -m venv .venv
              source .venv/bin/activate
              pip install -U pip setuptools wheel
              pip install -r tests/requirements.txt
            '';
            checkPhase = ''
              source .venv/bin/activate
              python3 -m pytest tests/
            '';
            doCheck = true;
          };
          buildDevShell = extraPackages: craneLib.devShell {
            packages = with pkgs; [
              # testing utilities
              jq
              yq-go
              curl

              # test Github automation
              act

              # documentation utilities
              pandoc

              # docker image inspection and handling
              dive
              skopeo

              # test coverage generation
              lcov

              # solidity development and chain interaction
              foundry-bin

              ## python is required by integration tests
              python39
              python39Packages.venvShellHook
            ] ++
            lib.optionals stdenv.isLinux [ autoPatchelfHook ] ++ extraPackages;
            venvDir = "./.venv";
            postVenvCreation = ''
              unset SOURCE_DATE_EPOCH
              pip install -U pip setuptools wheel
              pip install -r tests/requirements.txt
            '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              autoPatchelf ./.venv
            '';
          };
          defaultDevShell = buildDevShell [ ];
          smoketestsDevShell = buildDevShell [ hoprd hopli ];
        in
        {
          apps = {
            inherit hoprd-docker-build-and-upload hopli-docker-build-and-upload;
          };
          packages = {
            inherit hoprd hoprd-test hoprd-docker;
            inherit hopli hopli-test hopli-docker;
            inherit anvil-docker;
            inherit smoke-tests docs;
            default = hoprd;
          };
          devShells.default = defaultDevShell;
          devShells.smoke-tests = smoketestsDevShell;
        };
      systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin" ];
    };
}
