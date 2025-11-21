{
  description = "hoprnet monorepo";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.21.0";
    nix-lib.url = "github:hoprnet/nix-lib";
    # pin it to a version which we are compatible with
    foundry.url = "github:hoprnet/foundry.nix/tb/202505-add-xz";
    solc.url = "github:hellwolf/solc.nix";
    pre-commit.url = "github:cachix/git-hooks.nix";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-root.url = "github:srid/flake-root";

    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    foundry.inputs.flake-utils.follows = "flake-utils";
    foundry.inputs.nixpkgs.follows = "nixpkgs";
    nix-lib.inputs.nixpkgs.follows = "nixpkgs";
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
      nix-lib,
      foundry,
      # solc,
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
            # solc.overlay
          ];
          pkgs = import nixpkgs { inherit localSystem overlays; };
          buildPlatform = pkgs.stdenv.buildPlatform;
          # solcDefault = solc.mkDefault pkgs pkgs.solc_0_8_19;

          # Import nix-lib for shared Nix utilities
          nixLib = nix-lib.lib.${system};

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

          # Use nix-lib's source filtering for better rebuild performance
          depsSrc = nixLib.mkDepsSrc {
            root = ./.;
            inherit fs;
          };
          src = nixLib.mkSrc {
            root = ./.;
            inherit fs;
            extraFiles = [
              ./hoprd/hoprd/example_cfg.yaml
            ];
          };
          testSrc = nixLib.mkTestSrc {
            root = ./.;
            inherit fs;
            extraFiles = [
              ./hopr/hopr-lib/tests
              ./hoprd/hoprd/example_cfg.yaml
            ];
          };

          # Use nix-lib to create all rust builders for cross-compilation
          builders = nixLib.mkRustBuilders {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain.toml;
          };

          # Convenience aliases for builders
          rust-builder-local = builders.local;
          rust-builder-x86_64-linux = builders.x86_64-linux;
          rust-builder-x86_64-darwin = builders.x86_64-darwin;
          rust-builder-aarch64-linux = builders.aarch64-linux;
          rust-builder-aarch64-darwin = builders.aarch64-darwin;

          # Nightly builder for docs and specific features
          rust-builder-local-nightly = nixLib.mkRustBuilder {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain.toml;
            useRustNightly = true;
          };

          hoprdBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p hoprd-api -F allocator-jemalloc";
            cargoToml = ./hoprd/hoprd/Cargo.toml;
          };

          hoprd = rust-builder-local.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          # also used for Docker image
          hoprd-x86_64-linux = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          # also used for Docker image
          hoprd-x86_64-linux-profile = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );
          # also used for Docker image
          hoprd-x86_64-linux-dev = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs
            // {
              CARGO_PROFILE = "dev";
              cargoExtraArgs = "-F capture";
            }
          );
          hoprd-aarch64-linux = rust-builder-aarch64-linux.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          hoprd-aarch64-linux-profile = rust-builder-aarch64-linux.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );

          # CAVEAT: must be built from a darwin system
          hoprd-x86_64-darwin = rust-builder-x86_64-darwin.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          hoprd-x86_64-darwin-profile = rust-builder-x86_64-darwin.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );
          # CAVEAT: must be built from a darwin system
          hoprd-aarch64-darwin = rust-builder-aarch64-darwin.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          hoprd-aarch64-darwin-profile = rust-builder-aarch64-darwin.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { cargoExtraArgs = "-F capture"; }
          );

          hopr-test-unit = rust-builder-local.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs
            // {
              src = testSrc;
              runTests = true;
              cargoExtraArgs = "--lib";
            }
          );

          hopr-test-nightly = rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs
            // {
              src = testSrc;
              runTests = true;
              cargoExtraArgs = "-Z panic-abort-tests --lib";
            }
          );

          hoprd-clippy = rust-builder-local.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { runClippy = true; }
          );
          hoprd-dev = rust-builder-local.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs
            // {
              CARGO_PROFILE = "dev";
              cargoExtraArgs = "-F capture";
            }
          );
          # build candidate binary as static on Linux amd64 to get more test exposure specifically via smoke tests
          mkHoprdCandidate =
            cargoExtraArgs:
            if buildPlatform.isLinux && buildPlatform.isx86_64 then
              rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (
                hoprdBuildArgs
                // {
                  inherit cargoExtraArgs;
                  CARGO_PROFILE = "candidate";
                }
              )
            else
              rust-builder-local.callPackage nixLib.mkRustPackage (
                hoprdBuildArgs
                // {
                  inherit cargoExtraArgs;
                  CARGO_PROFILE = "candidate";
                }
              );
          # Use cross-compilation environment when possible to have the same setup as our production builds when benchmarking.
          hoprd-bench =
            if buildPlatform.isLinux && buildPlatform.isx86_64 then
              rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (hoprdBuildArgs // { runBench = true; })
            else if buildPlatform.isLinux && buildPlatform.isAarch64 then
              rust-builder-aarch64-linux.callPackage nixLib.mkRustPackage (hoprdBuildArgs // { runBench = true; })
            else if buildPlatform.isDarwin && buildPlatform.isx86_64 then
              rust-builder-x86_64-darwin.callPackage nixLib.mkRustPackage (hoprdBuildArgs // { runBench = true; })
            else if buildPlatform.isDarwin && buildPlatform.isAarch64 then
              rust-builder-aarch64-darwin.callPackage nixLib.mkRustPackage (
                hoprdBuildArgs // { runBench = true; }
              )
            else
              rust-builder-local.callPackage nixLib.mkRustPackage (hoprdBuildArgs // { runBench = true; });

          hopliBuildArgs = {
            inherit src depsSrc rev;
            cargoToml = ./hopli/Cargo.toml;
            # postInstall = ''
            #   mkdir -p $out/ethereum/contracts
            #   cp ethereum/contracts/contracts-addresses.json $out/ethereum/contracts/
            # '';
          };

          hopli = rust-builder-local.callPackage nixLib.mkRustPackage hopliBuildArgs;
          # also used for Docker image
          hopli-x86_64-linux = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage hopliBuildArgs;
          # also used for Docker image
          hopli-x86_64-linux-dev = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (
            hopliBuildArgs // { CARGO_PROFILE = "dev"; }
          );
          hopli-aarch64-linux = rust-builder-aarch64-linux.callPackage nixLib.mkRustPackage hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-x86_64-darwin = rust-builder-x86_64-darwin.callPackage nixLib.mkRustPackage hopliBuildArgs;
          # CAVEAT: must be built from a darwin system
          hopli-aarch64-darwin = rust-builder-aarch64-darwin.callPackage nixLib.mkRustPackage hopliBuildArgs;

          hopli-clippy = rust-builder-local.callPackage nixLib.mkRustPackage (
            hopliBuildArgs // { runClippy = true; }
          );

          hopli-dev = rust-builder-local.callPackage nixLib.mkRustPackage (
            hopliBuildArgs // { CARGO_PROFILE = "dev"; }
          );
          profileDeps = with pkgs; [
            gdb
            # FIXME: heaptrack would be useful, but it adds 700MB to the image size (unpacked)
            # lldb
            rust-bin.stable.latest.minimal
            valgrind

            # Networking tools to debug network issues
            tcpdump
            iproute2
            netcat
            iptables
            bind
            curl
            iputils
            nmap
            nethogs
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
              rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage (
                hopliBuildArgs // { CARGO_PROFILE = "candidate"; }
              )
            else
              rust-builder-local.callPackage nixLib.mkRustPackage (
                hopliBuildArgs // { CARGO_PROFILE = "candidate"; }
              );

          # Man pages using nix-lib
          hoprd-man = nixLib.mkManPage {
            pname = "hoprd";
            binary = hoprd-dev;
            description = "HOPR node executable";
          };
          hopli-man = nixLib.mkManPage {
            pname = "hopli";
            binary = hopli-dev;
            description = "HOPR CLI helper tool";
          };

          # FIXME: the docker image built is not working on macOS arm platforms
          # and will simply lead to a non-working image. Likely, some form of
          # cross-compilation or distributed build is required.
          # Docker images using nix-lib
          hoprd-docker = nixLib.mkDockerImage {
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              hoprd-x86_64-linux
            ];
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
          };
          hoprd-dev-docker = nixLib.mkDockerImage {
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              hoprd-x86_64-linux-dev
            ];
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
          };
          hoprd-profile-docker = nixLib.mkDockerImage {
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              hoprd-x86_64-linux-profile
            ]
            ++ profileDeps;
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
          };

          hopli-docker = nixLib.mkDockerImage {
            name = "hopli";
            extraContents = [ hopli-x86_64-linux ];
            Entrypoint = [ "/bin/hopli" ];
            env = [
              "ETHERSCAN_API_KEY=placeholder"
              # "HOPLI_CONTRACTS_ROOT=${package}/ethereum/contracts"
            ];
          };
          hopli-dev-docker = nixLib.mkDockerImage {
            name = "hopli";
            extraContents = [ hopli-x86_64-linux-dev ];
            Entrypoint = [ "/bin/hopli" ];
            env = [
              "ETHERSCAN_API_KEY=placeholder"
            ];
          };
          hopli-profile-docker = nixLib.mkDockerImage {
            name = "hopli";
            extraContents = [ hopli-x86_64-linux ] ++ profileDeps;
            Entrypoint = [ "/bin/hopli" ];
            env = [
              "ETHERSCAN_API_KEY=placeholder"
            ];
          };

          # Docker security scanning and SBOM generation using nix-lib
          hoprd-docker-trivy = nixLib.mkTrivyScan {
            image = hoprd-docker;
            imageName = "hoprd";
          };
          hoprd-docker-sbom = nixLib.mkSBOM {
            image = hoprd-docker;
            imageName = "hoprd";
          };
          hopli-docker-trivy = nixLib.mkTrivyScan {
            image = hopli-docker;
            imageName = "hopli";
          };
          hopli-docker-sbom = nixLib.mkSBOM {
            image = hopli-docker;
            imageName = "hopli";
          };

          # Multi-arch Docker manifests using nix-lib
          # NOTE: These require images for both amd64 and arm64 to be pushed to a registry first
          # hoprd-docker-multiarch = nixLib.mkMultiArchManifest {
          #   name = "hoprd";
          #   tag = "latest";
          #   images = [
          #     { arch = "amd64"; digest = "sha256:..."; }
          #     { arch = "arm64"; digest = "sha256:..."; }
          #   ];
          # };

          # anvilSrc = fs.toSource {
          #   root = ./.;
          #   fileset = fs.unions [
          #     ./ethereum/contracts/contracts-addresses.json
          #     ./ethereum/contracts/foundry.in.toml
          #     ./ethereum/contracts/remappings.txt
          #     ./ethereum/contracts/Makefile
          #     ./scripts/run-local-anvil.sh
          #     ./scripts/utils.sh
          #     ./Makefile
          #     (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/script)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/test)
          #   ];
          # };
          # anvil-docker = pkgs.dockerTools.buildLayeredImage {
          #   name = "hopr-anvil";
          #   tag = "latest";
          #   # breaks binary reproducibility, but makes usage easier
          #   created = "now";
          #   contents = with pkgs; [
          #     anvilSrc
          #     coreutils
          #     curl
          #     findutils
          #     foundry-bin
          #     gnumake
          #     jq
          #     lsof
          #     runtimeShellPackage
          #     solcDefault
          #     time
          #     tini
          #     which
          #   ];
          #   enableFakechroot = true;
          #   fakeRootCommands = ''
          #     #!${pkgs.runtimeShell}

          #     # must generate the foundry.toml here
          #     if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
          #       echo "solc = \"${solcDefault}/bin/solc\""
          #       echo "Generating foundry.toml file!"
          #       sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
          #         /ethereum/contracts/foundry.in.toml >| \
          #         /ethereum/contracts/foundry.toml
          #     else
          #       echo "foundry.toml file already exists!"
          #     fi

          #     # rewrite remappings to use absolute paths to fix solc checks
          #     sed -i \
          #       's|../../vendor/|/vendor/|g' \
          #       /ethereum/contracts/remappings.txt

          #     # unlink all linked files/directories, because forge does
          #     # not work well with those
          #     cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
          #     rm -rf /ethereum/contracts
          #     mv /ethereum/contracts-unlinked /ethereum/contracts

          #     # need to point to the contracts directory for forge to work
          #     ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts
          #   '';
          #   config = {
          #     Cmd = [
          #       "/bin/tini"
          #       "--"
          #       "bash"
          #       "/scripts/run-local-anvil.sh"
          #     ];
          #     ExposedPorts = {
          #       "8545/tcp" = { };
          #     };
          #   };
          # };
          # plutoSrc = fs.toSource {
          #   root = ./.;
          #   fileset = fs.unions [
          #     ./ethereum/contracts/contracts-addresses.json
          #     ./ethereum/contracts/foundry.in.toml
          #     ./ethereum/contracts/remappings.txt
          #     ./ethereum/contracts/Makefile
          #     ./scripts/protocol-config-anvil.json
          #     ./scripts/run-local-anvil.sh
          #     ./scripts/run-local-cluster.sh
          #     ./scripts/utils.sh
          #     (fs.fileFilter (file: true) ./sdk)
          #     ./pyproject.toml
          #     ./tests/pyproject.toml
          #     ./Makefile
          #     (fs.fileFilter (file: file.hasExt "sol") ./vendor/solidity)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/src)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/script)
          #     (fs.fileFilter (file: file.hasExt "sol") ./ethereum/contracts/test)
          #   ];
          # };
          # plutoDeps = with pkgs; [
          #   curl
          #   foundry-bin
          #   gnumake
          #   hoprd
          #   hopli
          #   plutoSrc
          #   python313
          #   runtimeShellPackage
          #   solcDefault
          #   lsof
          #   tini
          #   uv
          #   which
          # ];
          # pluto-docker = import ./nix/docker-builder.nix {
          #   name = "hopr-pluto";
          #   pkgs = pkgs;
          #   extraContents = plutoDeps;
          #   extraPorts = {
          #     "3001-3006/tcp" = { };
          #     "10001-10101/tcp" = { };
          #     "10001-10101/udp" = { };
          #   };
          #   Cmd = [
          #     "/bin/tini"
          #     "--"
          #     "bash"
          #     "/scripts/run-local-cluster.sh"
          #   ];
          #   fakeRootCommands = ''
          #     #!${pkgs.runtimeShell}

          #     # must generate the foundry.toml here
          #     if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
          #       echo "solc = \"${solcDefault}/bin/solc\""
          #       echo "Generating foundry.toml file!"
          #       sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
          #         /ethereum/contracts/foundry.in.toml >| \
          #         /ethereum/contracts/foundry.toml
          #     else
          #       echo "foundry.toml file already exists!"
          #     fi

          #     # rewrite remappings to use absolute paths to fix solc checks
          #     sed -i \
          #       's|../../vendor/|/vendor/|g' \
          #       /ethereum/contracts/remappings.txt

          #     # unlink all linked files/directories, because forge does
          #     # not work well with those
          #     cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
          #     rm -rf /ethereum/contracts
          #     mv /ethereum/contracts-unlinked /ethereum/contracts

          #     # need to point to the contracts directory for forge to work
          #     ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts

          #     mkdir /tmp/
          #   '';
          # };

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
          # hopr-pluto-docker-build-and-upload = flake-utils.lib.mkApp {
          #   drv = dockerImageUploadScript pluto-docker;
          # };
          docs = rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { buildDocs = true; }
          );
          smoke-tests = pkgs.stdenv.mkDerivation {
            pname = "hoprd-smoke-tests";
            version = hoprdCrateInfo.version;
            src = fs.toSource {
              root = ./.;
              fileset = fs.unions [
                ./tests
                ./scripts
                ./sdk/python
              ];
            };
            buildInputs = with pkgs; [
              uv
              foundry-bin
              # solcDefault
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
            HOPR_INTERNAL_TRANSPORT_ACCEPT_PRIVATE_NETWORK_IP_ADDRESSES = "true"; # Allow local private IPs in smoke tests
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
              ".gcloudignore"
            ];
          };

          # check-bindings =
          #   { pkgs, solcDefault, ... }:
          #   pkgs.stdenv.mkDerivation {
          #     pname = "check-bindings";
          #     version = hoprdCrateInfo.version;

          #     src = ./.;

          #     buildInputs = with pkgs; [
          #       diffutils
          #       foundry-bin
          #       solcDefault
          #       just
          #     ];

          #     preConfigure = ''
          #       mkdir -p ethereum/contracts
          #       sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
          #         ${./ethereum/contracts/foundry.in.toml} > ./ethereum/contracts/foundry.toml
          #     '';

          #     buildPhase = ''
          #       just generate-bindings
          #     '';

          #     checkPhase = ''
          #       echo "Checking if generated bindings introduced changes..."
          #       if [ -d "ethereum/bindings/src/reference" ]; then
          #           echo "Generated bindings are outdated. Please run the binding generation and commit the changes."
          #           exit 1
          #       fi
          #       echo "Bindings are up to date."
          #     '';

          #     # Disable the installPhase
          #     installPhase = "mkdir -p $out";
          #     doCheck = true;
          #   };
          # Development shells using nix-lib
          devShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR Development";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              sqlite
              cargo-machete
              foundry-bin
              nfpm
              envsubst
            ];
            shellHook = ''
              ${pre-commit-check.shellHook}
            '';
          };

          ciShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR CI";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              act
              gh
              google-cloud-sdk
              cargo-machete
              graphviz
              swagger-codegen3
              vacuum-go
              zizmor
              gnupg
              perl
            ];
          };

          testShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR Testing";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              uv
              python313
              foundry-bin
            ];
            shellHook = ''
              uv sync --frozen
              unset SOURCE_DATE_EPOCH
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux "autoPatchelf ./.venv"}
            '';
          };

          ciTestDevShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR CI Test (Dev)";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              uv
              python313
              foundry-bin
              hoprd-dev
              hopli-dev
            ];
            shellHook = ''
              uv sync --frozen
              unset SOURCE_DATE_EPOCH
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux "autoPatchelf ./.venv"}
            '';
          };

          ciTestShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR CI Test (Candidate)";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              uv
              python313
              foundry-bin
              (mkHoprdCandidate "")
              hopli-candidate
            ];
            shellHook = ''
              uv sync --frozen
              unset SOURCE_DATE_EPOCH
              ${pkgs.lib.optionalString pkgs.stdenv.isLinux "autoPatchelf ./.venv"}
            '';
          };

          docsShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR Documentation";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              html-tidy
              pandoc
              sqlite
              cargo-machete
            ];
            shellHook = ''
              ${pre-commit-check.shellHook}
            '';
            rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
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
              "hopr/hopr-lib/tests/snapshots/*"
              "hoprd/.dockerignore"
              "hoprd/rest-api/.cargo/config"
              "nix/setup-hook-darwin.sh"
              "target/*"
              "tests/pytest.ini"
            ];

            programs.shfmt.enable = true;
            settings.formatter.shfmt.includes = [
              "*.sh"
              "deploy/compose/.env.sample"
              "deploy/compose/.env-secrets.sample"
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
            ];
            settings.formatter.prettier.excludes = [
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
            #   settings.formatter.solc = {
            #     command = "sh";
            #     options = [
            #       "-euc"
            #       ''
            #         # must generate the foundry.toml here, since this step could
            #         # be executed in isolation
            #         if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
            #           echo "solc = \"${solcDefault}/bin/solc\""
            #           echo "Generating foundry.toml file!"
            #           sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
            #             ethereum/contracts/foundry.in.toml >| \
            #             ethereum/contracts/foundry.toml
            #         else
            #           echo "foundry.toml file already exists!"
            #         fi

            #         for file in "$@"; do
            #           ${pkgs.foundry-bin}/bin/forge fmt $file \
            #             --root ./ethereum/contracts;
            #         done
            #       ''
            #       "--"
            #     ];
            #     includes = [ "*.sol" ];
            #   };
          };

          checks = {
            inherit hoprd-clippy hopli-clippy;
            # check-bindings = check-bindings {
            #   pkgs = pkgs;
            #   solcDefault = solcDefault;
            # };
          };

          apps = {
            inherit hoprd-docker-build-and-upload;
            inherit hoprd-dev-docker-build-and-upload;
            inherit hoprd-profile-docker-build-and-upload;
            inherit hopli-docker-build-and-upload;
            inherit hopli-dev-docker-build-and-upload;
            inherit hopli-profile-docker-build-and-upload;
            # inherit hopr-pluto-docker-build-and-upload;
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
            inherit hopli-candidate;
            inherit hopr-test-unit hopr-test-nightly;
            # inherit anvil-docker pluto-docker;
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
            hoprd-candidate = (mkHoprdCandidate "");
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
