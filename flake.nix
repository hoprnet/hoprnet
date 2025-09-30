{
  description = "hoprnet monorepo";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.21.0";
    # pin it to a version which we are compatible with
    foundry.url = "github:shazow/foundry.nix/be409169ca05954e28cfd6206934bdaffe695c4a";
    solc.url = "github:hellwolf/solc.nix";
    pre-commit.url = "github:cachix/git-hooks.nix";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-root.url = "github:srid/flake-root";

    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    foundry.inputs.flake-utils.follows = "flake-utils";
    foundry.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    solc.inputs.flake-utils.follows = "flake-utils";
    solc.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      flake-parts,
      rust-overlay,
      crane,
      foundry,
      solc,
      pre-commit,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      perSystem =
        {
          config,
          lib,
          system,
          ...
        }:
        let
          rev = toString (self.shortRev or self.dirtyShortRev);
          fs = lib.fileset;
          localSystem = system;
          overlays = [
            (import rust-overlay)
            foundry.overlay
            solc.overlay
          ];
          pkgs = import nixpkgs { inherit localSystem overlays; };
          buildPlatform = pkgs.stdenv.buildPlatform;
          solcDefault = solc.mkDefault pkgs pkgs.solc_0_8_30;
          craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
          hoprdCrateInfoOriginal = craneLib.crateNameFromCargoToml {
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          hoprdCrateInfo = {
            pname = "hoprd";
            # normalize the version to not include any suffixes so the cache
            # does not get busted
            version = pkgs.lib.strings.concatStringsSep "." (
              pkgs.lib.lists.take 3 (builtins.splitVersion hoprdCrateInfoOriginal.version)
            );
          };
          depsSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./.cargo/config.toml
              ./Cargo.lock
              (fs.fileFilter (file: file.name == "Cargo.toml") ./.)
            ];
          };
          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              ./hopr/hopr-lib/data
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/foundry.in.toml
              ./ethereum/contracts/remappings.txt
              ./hoprd/hoprd/example_cfg.yaml
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
            ];
          };
          testSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./.cargo/config.toml
              ./Cargo.lock
              ./README.md
              ./hopr/hopr-lib/data
              ./hopr/hopr-lib/tests
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/foundry.in.toml
              ./ethereum/contracts/remappings.txt
              ./hoprd/hoprd/example_cfg.yaml
              (fs.fileFilter (file: file.hasExt "rs") ./.)
              (fs.fileFilter (file: file.hasExt "toml") ./.)
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
            ];
          };

          rust-builder-local = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
          };

          rust-builder-local-nightly = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
            useRustNightly = true;
          };

          rust-builder-x86_64-linux = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
            crossSystem = pkgs.lib.systems.examples.musl64;
            isCross = true;
            isStatic = true;
          };

          rust-builder-x86_64-darwin = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
            crossSystem = pkgs.lib.systems.examples.x86_64-darwin;
            isCross = true;
          };

          rust-builder-aarch64-linux = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform-musl;
            isCross = true;
            isStatic = true;
          };

          rust-builder-aarch64-darwin = import ./nix/rust-builder.nix {
            inherit
              nixpkgs
              rust-overlay
              crane
              foundry
              solc
              localSystem
              ;
            crossSystem = pkgs.lib.systems.examples.aarch64-darwin;
            isCross = true;
          };

          hoprdBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p hoprd-api";
            cargoToml = ./hoprd/hoprd/Cargo.toml;
          };

          hoprd = rust-builder-local.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # also used for Docker image
          hoprd-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          # also used for Docker image
          hoprd-x86_64-linux-profile = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );
          # also used for Docker image
          hoprd-x86_64-linux-dev = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs
            // {
              CARGO_PROFILE = "dev";
              cargoExtraArgs = "-F capture";
            }
          );
          hoprd-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-aarch64-linux-profile = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );

          # CAVEAT: must be built from a darwin system
          hoprd-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-x86_64-darwin-profile = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );
          # CAVEAT: must be built from a darwin system
          hoprd-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix hoprdBuildArgs;
          hoprd-aarch64-darwin-profile = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );

          hopr-test = rust-builder-local.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs
            // {
              src = testSrc;
              runTests = true;
            }
          );

          hopr-test-nightly = rust-builder-local-nightly.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs
            // {
              src = testSrc;
              runTests = true;
              cargoExtraArgs = "-Z panic-abort-tests";
            }
          );

          hoprd-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { runClippy = true; }
          );
          hoprd-dev = rust-builder-local.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs
            // {
              CARGO_PROFILE = "dev";
              cargoExtraArgs = "-F capture";
            }
          );
          # build candidate binary as static on Linux amd64 to get more test exposure specifically via smoke tests
          hoprd-candidate =
            if buildPlatform.isLinux && buildPlatform.isx86_64 then
              rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { CARGO_PROFILE = "candidate"; }
              )
            else
              rust-builder-local.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { CARGO_PROFILE = "candidate"; }
              );
          # Use cross-compilation environment when possible to have the same setup as our production builds when benchmarking.
          hoprd-bench =
            if buildPlatform.isLinux && buildPlatform.isx86_64 then
              rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { runBench = true; }
              )
            else if buildPlatform.isLinux && buildPlatform.isAarch64 then
              rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { runBench = true; }
              )
            else if buildPlatform.isDarwin && buildPlatform.isx86_64 then
              rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { runBench = true; }
              )
            else if buildPlatform.isDarwin && buildPlatform.isAarch64 then
              rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix (
                hoprdBuildArgs // { runBench = true; }
              )
            else
              rust-builder-local.callPackage ./nix/rust-package.nix (hoprdBuildArgs // { runBench = true; });

          hopliBuildArgs = {
            inherit src depsSrc rev;
            cargoToml = ./hopli/Cargo.toml;
            postInstall = ''
              mkdir -p $out/ethereum/contracts
              cp ethereum/contracts/contracts-addresses.json $out/ethereum/contracts/
            '';
          };

          hopli = rust-builder-local.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # also used for Docker image
          hopli-x86_64-linux = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # also used for Docker image
          hopli-x86_64-linux-dev = rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
            hopliBuildArgs // { CARGO_PROFILE = "dev"; }
          );
          hopli-aarch64-linux = rust-builder-aarch64-linux.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-x86_64-darwin = rust-builder-x86_64-darwin.callPackage ./nix/rust-package.nix hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-aarch64-darwin = rust-builder-aarch64-darwin.callPackage ./nix/rust-package.nix hopliBuildArgs;

          hopli-clippy = rust-builder-local.callPackage ./nix/rust-package.nix (
            hopliBuildArgs // { runClippy = true; }
          );

          hopli-dev = rust-builder-local.callPackage ./nix/rust-package.nix (
            hopliBuildArgs // { CARGO_PROFILE = "dev"; }
          );
          profileDeps = with pkgs; [
            gdb
            # FIXME: heaptrack would be useful, but it adds 700MB to the image size (unpacked)
            # lldb
            rust-bin.stable.latest.minimal
            valgrind
            gnutar # Used to extract the pcap file from the docker container
          ];

          dockerHoprdEntrypoint = pkgs.writeShellScriptBin "docker-entrypoint.sh" ''
            set -euo pipefail

            # if the default listen host has not been set by the user,
            # we will set it to the container's ip address
            # defaulting to random port

            listen_host="''${HOPRD_DEFAULT_SESSION_LISTEN_HOST:-}"
            listen_host_preset_ip="''${listen_host%%:*}"
            listen_host_preset_port="''${listen_host#*:}"

            if [ -z "''${listen_host_preset_ip:-}" ]; then
              listen_host_ip="$(hostname -i)"

              if [ -z "''${listen_host_preset_port:-}" ]; then
                listen_host="''${listen_host_ip}:0"
              else
                listen_host="''${listen_host_ip}:''${listen_host_preset_port}"
              fi
            fi

            export HOPRD_DEFAULT_SESSION_LISTEN_HOST="''${listen_host}"

            if [ -n "''${1:-}" ] && [ -f "/bin/''${1:-}" ] && [ -x "/bin/''${1:-}" ]; then
              # allow execution of auxiliary commands
              exec "$@"
            else
              # default to hoprd
              exec /bin/hoprd "$@"
            fi
          '';

          # build candidate binary as static on Linux amd64 to get more test exposure specifically via smoke tests
          hopli-candidate =
            if buildPlatform.isLinux && buildPlatform.isx86_64 then
              rust-builder-x86_64-linux.callPackage ./nix/rust-package.nix (
                hopliBuildArgs // { CARGO_PROFILE = "candidate"; }
              )
            else
              rust-builder-local.callPackage ./nix/rust-package.nix (
                hopliBuildArgs // { CARGO_PROFILE = "candidate"; }
              );

          # Man pages
          man-pages = pkgs.callPackage ./nix/man-pages.nix {
            hoprd = hoprd-dev;
            hopli = hopli-dev;
          };
          hoprd-man = man-pages.hoprd-man;
          hopli-man = man-pages.hopli-man;

          # FIXME: the docker image built is not working on macOS arm platforms
          # and will simply lead to a non-working image. Likely, some form of
          # cross-compilation or distributed build is required.
          hoprdDockerArgs = package: deps: {
            inherit pkgs;
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              package
            ]
            ++ deps;
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
          };
          hoprd-docker = import ./nix/docker-builder.nix (hoprdDockerArgs hoprd-x86_64-linux [ ]);
          hoprd-dev-docker = import ./nix/docker-builder.nix (hoprdDockerArgs hoprd-x86_64-linux-dev [ ]);
          hoprd-profile-docker = import ./nix/docker-builder.nix (
            hoprdDockerArgs hoprd-x86_64-linux-profile profileDeps
          );

          hopliDockerArgs = package: deps: {
            inherit pkgs;
            name = "hopli";
            extraContents = [ package ] ++ deps;
            Entrypoint = [ "/bin/hopli" ];
            env = [
              "ETHERSCAN_API_KEY=placeholder"
              "HOPLI_CONTRACTS_ROOT=${package}/ethereum/contracts"
            ];
          };
          hopli-docker = import ./nix/docker-builder.nix (hopliDockerArgs hopli-x86_64-linux [ ]);
          hopli-dev-docker = import ./nix/docker-builder.nix (hopliDockerArgs hopli-x86_64-linux-dev [ ]);
          hopli-profile-docker = import ./nix/docker-builder.nix (
            hopliDockerArgs hopli-x86_64-linux profileDeps
          );

          anvilSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/foundry.in.toml
              ./ethereum/contracts/remappings.txt
              ./ethereum/contracts/Makefile
              ./scripts/run-local-anvil.sh
              ./scripts/utils.sh
              ./Makefile
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/script)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/test)
            ];
          };
          anvil-docker = pkgs.dockerTools.buildLayeredImage {
            name = "hopr-anvil";
            tag = "latest";
            # breaks binary reproducibility, but makes usage easier
            created = "now";
            contents = with pkgs; [
              anvilSrc
              coreutils
              curl
              findutils
              foundry-bin
              gnumake
              jq
              lsof
              runtimeShellPackage
              solcDefault
              time
              tini
              which
            ];
            enableFakechroot = true;
            fakeRootCommands = ''
              #!${pkgs.runtimeShell}

              # must generate the foundry.toml here
              if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
                echo "solc = \"${solcDefault}/bin/solc\""
                echo "Generating foundry.toml file!"
                sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                  /ethereum/contracts/foundry.in.toml >| \
                  /ethereum/contracts/foundry.toml
              else
                echo "foundry.toml file already exists!"
              fi

              # rewrite remappings to use absolute paths to fix solc checks
              sed -i \
                's|../../vendor/|/vendor/|g' \
                /ethereum/contracts/remappings.txt

              # unlink all linked files/directories, because forge does
              # not work well with those
              cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
              rm -rf /ethereum/contracts
              mv /ethereum/contracts-unlinked /ethereum/contracts

              # need to point to the contracts directory for forge to work
              ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts
            '';
            config = {
              Cmd = [
                "/bin/tini"
                "--"
                "bash"
                "/scripts/run-local-anvil.sh"
              ];
              ExposedPorts = {
                "8545/tcp" = { };
              };
            };
          };
          plutoSrc = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./ethereum/contracts/contracts-addresses.json
              ./ethereum/contracts/foundry.in.toml
              ./ethereum/contracts/remappings.txt
              ./ethereum/contracts/Makefile
              ./scripts/protocol-config-anvil.json
              ./scripts/run-local-anvil.sh
              ./scripts/run-local-cluster.sh
              ./scripts/utils.sh
              (fs.fileFilter (file: true) ./sdk)
              ./pyproject.toml
              ./tests/pyproject.toml
              ./Makefile
              (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/script)
              (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/test)
            ];
          };
          plutoDeps = with pkgs; [
            curl
            foundry-bin
            gnumake
            hoprd
            hopli
            plutoSrc
            python313
            runtimeShellPackage
            solcDefault
            lsof
            tini
            uv
            which
          ];
          pluto-docker = import ./nix/docker-builder.nix {
            name = "hopr-pluto";
            pkgs = pkgs;
            extraContents = plutoDeps;
            extraPorts = {
              "3001-3006/tcp" = { };
              "10001-10101/tcp" = { };
              "10001-10101/udp" = { };
            };
            Cmd = [
              "/bin/tini"
              "--"
              "bash"
              "/scripts/run-local-cluster.sh"
            ];
            fakeRootCommands = ''
              #!${pkgs.runtimeShell}

              # must generate the foundry.toml here
              if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
                echo "solc = \"${solcDefault}/bin/solc\""
                echo "Generating foundry.toml file!"
                sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                  /ethereum/contracts/foundry.in.toml >| \
                  /ethereum/contracts/foundry.toml
              else
                echo "foundry.toml file already exists!"
              fi

              # rewrite remappings to use absolute paths to fix solc checks
              sed -i \
                's|../../vendor/|/vendor/|g' \
                /ethereum/contracts/remappings.txt

              # unlink all linked files/directories, because forge does
              # not work well with those
              cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
              rm -rf /ethereum/contracts
              mv /ethereum/contracts-unlinked /ethereum/contracts

              # need to point to the contracts directory for forge to work
              ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts

              mkdir /tmp/
            '';
          };

          dockerImageUploadScript =
            image:
            pkgs.writeShellScriptBin "docker-image-upload" ''
              set -eu
              OCI_ARCHIVE="$(nix build --no-link --print-out-paths ${image})"
              ${pkgs.skopeo}/bin/skopeo copy --insecure-policy \
                --dest-registry-token="$GOOGLE_ACCESS_TOKEN" \
                "docker-archive:$OCI_ARCHIVE" "docker://$IMAGE_TARGET"
              echo "Uploaded image to $IMAGE_TARGET"
            '';
          hoprd-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hoprd-docker;
          };
          hoprd-dev-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hoprd-dev-docker;
          };
          hoprd-profile-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hoprd-profile-docker;
          };
          hopli-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hopli-docker;
          };
          hopli-dev-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hopli-dev-docker;
          };
          hopli-profile-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript hopli-profile-docker;
          };
          hopr-pluto-docker-build-and-upload = flake-utils.lib.mkApp {
            drv = dockerImageUploadScript pluto-docker;
          };
          docs = rust-builder-local-nightly.callPackage ./nix/rust-package.nix (
            hoprdBuildArgs // { buildDocs = true; }
          );
          smoke-tests = pkgs.stdenv.mkDerivation {
            pname = "hoprd-smoke-tests";
            version = hoprdCrateInfo.version;
            src = fs.toSource {
              root = ./.;
              fileset = fs.unions [
                (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
                ./tests
                ./scripts
                ./sdk/python
                ./ethereum/contracts/foundry.in.toml
                ./ethereum/contracts/remappings.txt
              ];
            };
            buildInputs = with pkgs; [
              uv
              foundry-bin
              solcDefault
              python313
              hopli-dev
              hoprd-dev
            ];
            buildPhase = ''
              uv sync --frozen
              unset SOURCE_DATE_EPOCH
            '';
            checkPhase = ''
              uv run --frozen -m pytest tests/
            '';
            doCheck = true;
          };
          pre-commit-check = pre-commit.lib.${system}.run {
            src = ./.;
            hooks = {
              # https://github.com/cachix/git-hooks.nix
              treefmt.enable = false;
              treefmt.package = config.treefmt.build.wrapper;
              check-executables-have-shebangs.enable = true;
              check-shebang-scripts-are-executable.enable = true;
              check-case-conflicts.enable = true;
              check-symlinks.enable = true;
              check-merge-conflicts.enable = true;
              check-added-large-files.enable = true;
              commitizen.enable = true;
              immutable-files = {
                enable = false;
                name = "Immutable files - the files should not change";
                entry = "bash .github/scripts/immutable-files-check.sh";
                files = "";
                language = "system";
              };
            };
            tools = pkgs;
            excludes = [
              "vendor/"
              "ethereum/contracts/"
              "ethereum/bindings/src/codegen"
              ".gcloudignore"
            ];
          };

          check-bindings =
            { pkgs, solcDefault, ... }:
            pkgs.stdenv.mkDerivation {
              pname = "check-bindings";
              version = hoprdCrateInfo.version;

              src = ./.;

              buildInputs = with pkgs; [
                diffutils
                foundry-bin
                solcDefault
                just
              ];

              preConfigure = ''
                mkdir -p ethereum/contracts
                sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                  ${./ethereum/contracts/foundry.in.toml} > ./ethereum/contracts/foundry.toml
              '';

              buildPhase = ''
                just generate-bindings
              '';

              checkPhase = ''
                echo "Checking if generated bindings introduced changes..."
                if [ -d "ethereum/bindings/src/reference" ]; then
                    echo "Generated bindings are outdated. Please run the binding generation and commit the changes."
                    exit 1
                fi
                echo "Bindings are up to date."
              '';

              # Disable the installPhase
              installPhase = "mkdir -p $out";
              doCheck = true;
            };
          devShell = import ./nix/devShell.nix {
            inherit
              pkgs
              config
              crane
              pre-commit-check
              solcDefault
              ;
            extraPackages = with pkgs; [
              nfpm
              envsubst
            ];
          };
          ciShell = import ./nix/ciShell.nix { inherit pkgs config crane; };
          testShell = import ./nix/testShell.nix {
            inherit
              pkgs
              config
              crane
              solcDefault
              ;
          };
          ciTestDevShell = import ./nix/ciTestShell.nix {
            inherit
              pkgs
              config
              crane
              solcDefault
              ;
            hoprd = hoprd-dev;
            hopli = hopli-dev;
          };
          ciTestShell = import ./nix/ciTestShell.nix {
            inherit
              pkgs
              config
              crane
              solcDefault
              ;
            hoprd = hoprd-candidate;
            hopli = hopli-candidate;
          };
          docsShell = import ./nix/devShell.nix {
            inherit
              pkgs
              config
              crane
              pre-commit-check
              solcDefault
              ;
            extraPackages = with pkgs; [
              html-tidy
              pandoc
            ];
            useRustNightly = true;
          };
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
          run-audit = flake-utils.lib.mkApp {
            drv = pkgs.writeShellApplication {
              name = "audit";
              runtimeInputs = [
                pkgs.cargo
                pkgs.cargo-audit
              ];
              text = ''
                cargo audit
              '';
            };
          };

          find-port-ci = flake-utils.lib.mkApp {
            drv = pkgs.writeShellApplication {
              name = "find-port";
              text = ''
                ${pkgs.python3}/bin/python ./tests/find_port.py --min-port 3000 --max-port 4000 --skip 30
              '';
            };
          };
          update-github-labels = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "update-github-labels" ''
              set -eu
              # remove existing crate entries (to remove old crates)
              yq 'with_entries(select(.key != "crate:*"))' .github/labeler.yml > labeler.yml.new
              # add new crate entries for known crates
              for f in `find . -mindepth 2 -name "Cargo.toml" -type f -printf '%P\n'`; do
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

            settings.global.excludes = [
              "**/*.id"
              "**/.cargo-ok"
              "**/.gitignore"
              ".actrc"
              ".dockerignore"
              ".editorconfig"
              ".gcloudignore"
              ".gitattributes"
              ".yamlfmt"
              "LICENSE"
              "Makefile"
              "db/entity/src/codegen/*"
              "deploy/compose/grafana/config.monitoring"
              "deploy/nfpm/nfpm.yaml"
              ".github/workflows/build-binaries.yaml"
              "docs/*"
              "ethereum/bindings/src/codegen/*"
              "ethereum/contracts/Makefile"
              "ethereum/contracts/broadcast/*"
              "ethereum/contracts/contracts-addresses.json"
              "ethereum/contracts/remappings.txt"
              "ethereum/contracts/src/static/*"
              "ethereum/contracts/test/static/*"
              "hopr/hopr-lib/tests/snapshots/*"
              "hoprd/.dockerignore"
              "hoprd/rest-api/.cargo/config"
              "nix/setup-hook-darwin.sh"
              "target/*"
              "tests/pytest.ini"
              "vendor/*"
            ];

            programs.shfmt.enable = true;
            settings.formatter.shfmt.includes = [
              "*.sh"
              "deploy/compose/.env.sample"
              "deploy/compose/.env-secrets.sample"
              "ethereum/contracts/.env.example"
            ];

            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [
              ".github/labeler.yml"
              ".github/workflows/*.yaml"
            ];
            # trying setting from https://github.com/google/yamlfmt/blob/main/docs/config-file.md
            settings.formatter.yamlfmt.settings = {
              formatter.type = "basic";
              formatter.max_line_length = 120;
              formatter.trim_trailing_whitespace = true;
              formatter.scan_folded_as_literal = true;
              formatter.include_document_start = true;
            };

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [
              "*.md"
              "*.json"
              "ethereum/contracts/README.md"
            ];
            settings.formatter.prettier.excludes = [
              "ethereum/contracts/*"
              "*.yml"
              "*.yaml"
            ];
            programs.rustfmt.enable = true;
            # using the official Nixpkgs formatting
            # see https://github.com/NixOS/rfcs/blob/master/rfcs/0166-nix-formatting.md
            programs.nixfmt.enable = true;
            programs.taplo.enable = true;
            programs.ruff-format.enable = true;

            settings.formatter.rustfmt = {
              command = "${pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)}/bin/rustfmt";
            };
            settings.formatter.solc = {
              command = "sh";
              options = [
                "-euc"
                ''
                  # must generate the foundry.toml here, since this step could
                  # be executed in isolation
                  if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
                    echo "solc = \"${solcDefault}/bin/solc\""
                    echo "Generating foundry.toml file!"
                    sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
                      ethereum/contracts/foundry.in.toml >| \
                      ethereum/contracts/foundry.toml
                  else
                    echo "foundry.toml file already exists!"
                  fi

                  for file in "$@"; do
                    ${pkgs.foundry-bin}/bin/forge fmt $file \
                      --root ./ethereum/contracts;
                  done
                ''
                "--"
              ];
              includes = [ "*.sol" ];
            };
          };

          checks = {
            inherit hoprd-clippy hopli-clippy;
            check-bindings = check-bindings {
              pkgs = pkgs;
              solcDefault = solcDefault;
            };
          };

          apps = {
            inherit hoprd-docker-build-and-upload;
            inherit hoprd-dev-docker-build-and-upload;
            inherit hoprd-profile-docker-build-and-upload;
            inherit hopli-docker-build-and-upload;
            inherit hopli-dev-docker-build-and-upload;
            inherit hopli-profile-docker-build-and-upload;
            inherit hopr-pluto-docker-build-and-upload;
            inherit update-github-labels find-port-ci;
            check = run-check;
            audit = run-audit;
          };

          packages = {
            inherit
              hoprd
              hoprd-dev
              hoprd-docker
              hoprd-dev-docker
              hoprd-profile-docker
              ;
            inherit
              hopli
              hopli-dev
              hopli-docker
              hopli-dev-docker
              hopli-profile-docker
              ;
            inherit hoprd-candidate hopli-candidate;
            inherit hopr-test hopr-test-nightly;
            inherit anvil-docker pluto-docker;
            inherit smoke-tests docs;
            inherit pre-commit-check;
            inherit hoprd-bench;
            inherit hoprd-man hopli-man;
            # binary packages
            inherit hoprd-x86_64-linux hoprd-x86_64-linux-dev hoprd-x86_64-linux-profile;
            inherit hoprd-aarch64-linux hoprd-aarch64-linux-profile;
            inherit hopli-x86_64-linux hopli-x86_64-linux-dev;
            inherit hopli-aarch64-linux;
            # FIXME: Darwin cross-builds are currently broken.
            # Follow https://github.com/nixos/nixpkgs/pull/256590
            inherit hoprd-x86_64-darwin hoprd-x86_64-darwin-profile;
            inherit hoprd-aarch64-darwin hoprd-aarch64-darwin-profile;
            inherit hopli-x86_64-darwin;
            inherit hopli-aarch64-darwin;
            default = hoprd;
          };

          devShells.default = devShell;
          devShells.ci = ciShell;
          devShells.test = testShell;
          devShells.citest = ciTestShell;
          devShells.citestdev = ciTestDevShell;
          devShells.docs = docsShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [
        "x86_64-linux"
        # NOTE: blocked by missing support in solc, see
        # https://github.com/ethereum/solidity/issues/11351
        # "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
    };
}
