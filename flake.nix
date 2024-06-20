{
  description = "hoprnet monorepo";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    flake-parts.url = github:hercules-ci/flake-parts;
    nixpkgs.url = github:NixOS/nixpkgs/release-24.05;
    # using bugfix for macos libcurl:
    # https://github.com/oxalica/rust-overlay/pull/149
    rust-overlay.url = github:oxalica/rust-overlay/647bff9f5e10d7f1756d86eee09831e6b1b06430;
    # using a fork with an added source filter
    crane.url = github:hoprnet/crane/tb/20240117-find-filter;
    # pin it to a version which we are compatible with
    foundry.url = github:shazow/foundry.nix/ece7c960a440c6725a7a5576d1f49a5fabde3747;
    # use change to add solc 0.8.24
    solc.url = github:hoprnet/solc.nix/tb/20240129-solc-0.8.24;
    pre-commit.url = github:cachix/pre-commit-hooks.nix;
    treefmt-nix.url = github:numtide/treefmt-nix;
    flake-root.url = github:srid/flake-root;

    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    foundry.inputs.flake-utils.follows = "flake-utils";
    foundry.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs-stable.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    solc.inputs.flake-utils.follows = "flake-utils";
    solc.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, flake-parts, flake-root, rust-overlay, crane, foundry, solc, pre-commit, treefmt-nix, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      perSystem = { config, lib, self', inputs', system, ... }:
        let
          fs = lib.fileset;
          localSystem = system;
          overlays = [ (import rust-overlay) foundry.overlay solc.overlay ];
          pkgs = import nixpkgs {
            inherit localSystem overlays;
          };
          solcDefault = solc.mkDefault pkgs pkgs.solc_0_8_19;
          rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          craneLibNightly = (crane.mkLib pkgs).overrideToolchain rustNightly;
          hoprdCrateInfoOriginal = craneLibNightly.crateNameFromCargoToml {
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          hoprdCrateInfo = {
            pname = "hoprd";
            # normalize the version to not include any suffixes so the cache
            # does not get busted
            version = pkgs.lib.strings.concatStringsSep "."
              (pkgs.lib.lists.take 3 (builtins.splitVersion hoprdCrateInfoOriginal.version));
          };
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
              ./ethereum/contracts/foundry.toml.in
              ./ethereum/contracts/remappings.txt
              ./hoprd/hoprd/example_cfg.yaml
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
            ];
          };

          rust-builder-local = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
          };

          rust-builder-local-nightly = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            useRustNightly = true;
          };

          rust-builder-x86_64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.gnu64;
          };

          rust-builder-x86_64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.x86_64-darwin;
          };

          rust-builder-aarch64-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
          };

          rust-builder-aarch64-darwin = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.aarch64-darwin;
          };

          rust-builder-armv7l-linux = import ./nix/rust-builder.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
            crossSystem = pkgs.lib.systems.examples.armv7l-hf-multiplatform;
          };

          hoprdBuildArgs = {
            inherit src depsSrc;
            cargoToml = ./hoprd/hoprd/Cargo.toml;
          };

          hoprd = rust-builder-local.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-armv7l-linux = rust-builder-armv7l-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # CAVEAT: must be built from a darwin system
          hoprd-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # CAVEAT: must be built from a darwin system
          hoprd-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;

          hoprd-test = rust-builder-local.callPackage ./nix/rust-package.nix (hoprdBuildArgs // { runTests = true; });
          hoprd-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (hoprdBuildArgs // { runClippy = true; });
          hoprd-debug = rust-builder-local.callPackage ./nix/rust-package.nix (hoprdBuildArgs // {
            CARGO_PROFILE = "dev";
          });

          hopliBuildArgs = {
            inherit src depsSrc;
            cargoToml = ./hopli/Cargo.toml;
            postInstall = ''
              mkdir -p $out/ethereum/contracts
              cp ethereum/contracts/contracts-addresses.json $out/ethereum/contracts/
            '';
          };

          hopli = rust-builder-local.callPackage ./nix/rust-package.nix hopliBuildArgs;
          hopli-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix hopliBuildArgs;
          hopli-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix hopliBuildArgs;
          hopli-armv7l-linux = rust-builder-armv7l-linux.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix hopliBuildArgs;

          hopli-test = rust-builder-local.callPackage ./nix/rust-package.nix (hopliBuildArgs // { runTests = true; });
          hopli-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (hopliBuildArgs // { runClippy = true; });
          hopli-debug = rust-builder-local.callPackage ./nix/rust-package.nix (hopliBuildArgs // {
            CARGO_PROFILE = "dev";
          });

          # FIXME: the docker image built is not working on macOS arm platforms
          # and will simply lead to a non-working image. Likely, some form of
          # cross-compilation or distributed build is required.
          hoprd-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hoprd";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = with pkgs; [
              hoprd
              iana-etc
              cacert
              bash
              findutils
              coreutils
            ];
            config = {
              Entrypoint = [
                "/bin/hoprd"
              ];
              Env = [
                "NO_COLOR=true" # suppress colored log output
                # "RUST_LOG=info"   # 'info' level is set by default with some spamming components set to override
                "RUST_BACKTRACE=full"
              ];
            };
          };
          hopli-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hopli";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = with pkgs; [
              hopli
              iana-etc
              cacert
              bash
              findutils
              coreutils
            ];
            config = {
              Entrypoint = [
                "/bin/hopli"
              ];
              Env = [
                # "RUST_LOG=info"   # 'info' level is set by default with some spamming components set to override
                "RUST_BACKTRACE=full"
                "NO_COLOR=true" # suppress colored log output
                "ETHERSCAN_API_KEY=placeholder"
                "HOPLI_CONTRACTS_ROOT=${hopli}/ethereum/contracts"
              ];
            };
          };
          hoprd-debug-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hoprd-debug";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            # size ends up being ca. 450MB packed, 1.3GB unpacked
            contents = with pkgs; [
              cacert
              gdb
              # FIXME: would be useful, but 700MB larger image size (unpacked)
              # heaptrack
              hoprd
              iana-etc
              bash
              # FIXME: would be useful, but 1300MB larger image size (unpacked)
              # lldb
              rust-bin.stable.latest.minimal
              valgrind
            ];
            config = {
              Entrypoint = [
                "/bin/hoprd"
              ];
              Env = [
                "NO_COLOR=true" # suppress colored log output
                "RUST_LOG=debug,libp2p_mplex=info,multistream_select=info,isahc::handler=error,isahc::client=error"
                "RUST_BACKTRACE=full"
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
          hoprd-debug-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hoprd-debug-docker;
          };
          hopli-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hopli-docker;
          };
          docs = rust-builder-local-nightly.callPackage ./nix/rust-package.nix (hoprdBuildArgs // {
            buildDocs = true;
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
                ./ethereum/contracts/foundry.toml.in
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
          pre-commit-check = pre-commit.lib.${system}.run {
            src = ./.;
            hooks = {
              treefmt.enable = true;
              treefmt.package = config.treefmt.build.wrapper;
            };
            tools = pkgs;
          };
          defaultDevShell = import ./nix/shell.nix { inherit pkgs config crane pre-commit-check solcDefault; };
          smoketestsDevShell = import ./nix/shell.nix { inherit pkgs config crane pre-commit-check solcDefault; extraPackages = [ hoprd hopli ]; };
          run-check = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "run-check" ''
              set -e
              check=$1
              if [ -z "$check" ]; then
                nix flake show --json 2>/dev/null | \
                  jq -r '.checks."${system}" | to_entries | .[].key' | \
                  xargs -I '{}' nix build ".#checks."${system}".{}"
              else
              	nix build ".#checks."${system}".$check"
              fi
            '';
          };
          update-github-labels = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "update-github-labels" ''
              set -eu
              # remove existing crate entries (to remove old crates)
              yq 'with_entries(select(.key != "crate:*"))' .github/labeler.yml > labeler.yml.new
              # add new crate entries for known crates
              for f in `find . -mindepth 2 -name "Cargo.toml" -type f ! -path "./vendor/*" -printf '%P\n'`; do
              	env \
              		name="crate:`yq '.package.name' $f`" \
              		dir="`dirname $f`/**" \
              		yq -n '.[strenv(name)][0]."changed-files"[0]."any-glob-to-any-file" = env(dir)' >> labeler.yml.new
              done
              mv labeler.yml.new .github/labeler.yml
            '';
          };
        in
        {
          treefmt = {
            inherit (config.flake-root) projectRootFile;

            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [ "./.github/labeler.yml" "./.github/workflows/*.yaml" ];
            settings.formatter.yamlfmt.excludes = [ "./vendor/*" ];

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [ "*.md" "*.json" ];
            settings.formatter.prettier.excludes = [ "./vendor/*" "./ethereum/contracts/broadcast/*" "*.yml" "*.yaml" ];

            programs.rustfmt.enable = true;
            settings.formatter.rustfmt.excludes = [ "./vendor/*" ];

            programs.nixpkgs-fmt.enable = true;
            settings.formatter.nixpkgs-fmt.excludes = [ "./vendor/*" ];

            # FIXME: currently broken in treefmt
            # programs.ruff.check = true;
            # settings.formatter.ruff.check.excludes = [ "./vendor/*" ];

            settings.formatter.solc = {
              command = "sh";
              options = [
                "-euc"
                ''
                  # must generate the foundry.toml here, since this step could
                  # be executed in isolation
                  sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                    ./ethereum/contracts/foundry.toml.in > \
                    ./ethereum/contracts/foundry.toml

                  for file in "$@"; do
                    ${pkgs.foundry-bin}/bin/forge fmt $file \
                      --root ./ethereum/contracts;
                  done
                ''
                "--"
              ];
              includes = [ "*.sol" ];
              excludes = [ "./vendor/*" "./ethereum/contracts/src/static/*" ];
            };
          };

          checks = {
            inherit hoprd-clippy hopli-clippy;
          };

          apps = {
            inherit hoprd-docker-build-and-upload;
            inherit hoprd-debug-docker-build-and-upload;
            inherit hopli-docker-build-and-upload;
            inherit update-github-labels;
            check = run-check;
          };

          packages = {
            inherit hoprd hoprd-debug hoprd-test hoprd-docker hoprd-debug-docker;
            inherit hopli hopli-debug hopli-test hopli-docker;
            inherit anvil-docker;
            inherit smoke-tests docs;
            inherit pre-commit-check;
            inherit hoprd-aarch64-linux hoprd-armv7l-linux hoprd-x86_64-linux;
            inherit hopli-aarch64-linux hopli-armv7l-linux hopli-x86_64-linux;
            # FIXME: Darwin cross-builds are currently broken.
            # Follow https://github.com/nixos/nixpkgs/pull/256590
            inherit hoprd-aarch64-darwin hoprd-x86_64-darwin;
            inherit hopli-aarch64-darwin hopli-x86_64-darwin;
            default = hoprd;
          };

          devShells.default = defaultDevShell;
          devShells.smoke-tests = smoketestsDevShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
    };
}
