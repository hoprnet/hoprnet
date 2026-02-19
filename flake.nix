{
  description = "hopr-lib and hoprd repository";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/master";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.23.0";
    nix-lib.url = "github:hoprnet/nix-lib";
    # pin it to a version which we are compatible with
    foundry.url = "github:hoprnet/foundry.nix/tb/202505-add-xz";
    pre-commit.url = "github:cachix/git-hooks.nix";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-root.url = "github:srid/flake-root";
    hopli.url = "github:hoprnet/hopli";

    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    foundry.inputs.flake-utils.follows = "flake-utils";
    foundry.inputs.nixpkgs.follows = "nixpkgs";
    nix-lib.inputs.nixpkgs.follows = "nixpkgs";
    nix-lib.inputs.flake-utils.follows = "flake-utils";
    nix-lib.inputs.crane.follows = "crane";
    nix-lib.inputs.flake-parts.follows = "flake-parts";
    nix-lib.inputs.rust-overlay.follows = "rust-overlay";
    nix-lib.inputs.treefmt-nix.follows = "treefmt-nix";
    nix-lib.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    hopli.inputs.flake-utils.follows = "flake-utils";
    hopli.inputs.flake-parts.follows = "flake-parts";
    hopli.inputs.nixpkgs.follows = "nixpkgs";
    hopli.inputs.rust-overlay.follows = "rust-overlay";
    hopli.inputs.crane.follows = "crane";
    hopli.inputs.nix-lib.follows = "nix-lib";
    hopli.inputs.foundry.follows = "foundry";
    hopli.inputs.pre-commit.follows = "pre-commit";
    hopli.inputs.treefmt-nix.follows = "treefmt-nix";
    hopli.inputs.flake-root.follows = "flake-root";
    hopli.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
      flake-parts,
      rust-overlay,
      crane,
      nix-lib,
      foundry,
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
          ];
          pkgs = import nixpkgs { inherit localSystem overlays; };
          pkgs-unstable = import nixpkgs-unstable { inherit localSystem overlays; };
          buildPlatform = pkgs.stdenv.buildPlatform;

          # Import nix-lib for shared Nix utilities
          nixLib = nix-lib.lib.${system};

          # Load nightly toolchain from file (single source of truth)
          nightlyToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain-nightly.toml;

          # Wrapper for rustfmt to fix macOS dylib loading issue
          # On macOS, rust-overlay symlinks rustfmt to a standalone package that can't find its dylibs.
          # This wrapper sets DYLD_LIBRARY_PATH to the toolchain's lib directory.
          rustfmtWrapper = pkgs.writeShellScriptBin "rustfmt" ''
            export DYLD_LIBRARY_PATH="${nightlyToolchain}/lib:$DYLD_LIBRARY_PATH"
            exec "${nightlyToolchain}/bin/rustfmt" "$@"
          '';

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
            extraFiles = [ ./hoprd/hoprd/example_cfg.yaml ];
          };
          testSrc = nixLib.mkTestSrc {
            root = ./.;
            inherit fs;
            extraFiles = [
              ./hopr/builder/tests
              ./hoprd/hoprd/example_cfg.yaml
              (fs.fileFilter (file: file.hasExt "snap") ./.)
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
          # Uses a pinned nightly version to avoid ICE bugs in latest nightly
          rust-builder-local-nightly = nixLib.mkRustBuilder {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain-nightly.toml;
          };

          hoprdBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p hoprd-api -F allocator-jemalloc";
            cargoToml = ./hoprd/hoprd/Cargo.toml;
          };
          localclusterBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p hoprd-localcluster";
            cargoToml = ./localcluster/Cargo.toml;
          };

          hoprd = rust-builder-local.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          hoprd-localcluster = rust-builder-local.callPackage nixLib.mkRustPackage localclusterBuildArgs;
          # also used for Docker image
          hoprd-x86_64-linux = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage hoprdBuildArgs;
          # also used for Docker image
          hoprd-localcluster-x86_64-linux = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage localclusterBuildArgs;
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

            # ensure TLS clients can locate the CA bundle inside the container
            ssl_cert_file="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            if [ -f "$ssl_cert_file" ]; then
              export SSL_CERT_FILE="$ssl_cert_file"
              export NIX_SSL_CERT_FILE="$ssl_cert_file"
            fi

            # ensure the temporary directory exists
            mkdir -p $TMPDIR

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
          # Man pages using nix-lib
          hoprd-man = nixLib.mkManPage {
            pname = "hoprd";
            binary = hoprd-dev;
            description = "HOPR node executable";
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
              pkgs.cacert
              pkgs.curl # Required by docker-compose healthcheck
            ];
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
            env = [ "TMPDIR=/app/.tmp" ];
          };
          hoprd-dev-docker = nixLib.mkDockerImage {
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              hoprd-x86_64-linux-dev
              pkgs.cacert
              pkgs.curl # Required by docker-compose healthcheck
            ];
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
            env = [ "TMPDIR=/app/.tmp" ];
          };
          hoprd-profile-docker = nixLib.mkDockerImage {
            name = "hoprd";
            extraContents = [
              dockerHoprdEntrypoint
              hoprd-x86_64-linux-profile
              pkgs.cacert
              pkgs.curl # Required by docker-compose healthcheck
            ]
            ++ profileDeps;
            Entrypoint = [ "/bin/docker-entrypoint.sh" ];
            Cmd = [ "hoprd" ];
            env = [ "TMPDIR=/app/.tmp" ];
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
          docs = rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
            hoprdBuildArgs // { buildDocs = true; }
          );
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
            excludes = [ ".gcloudignore" ];
          };

          # Development shells using nix-lib
          devShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR Development";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              sqlite
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
              cargo-insta
              foundry-bin
              nfpm
              envsubst
            ];
            shellHook = ''
              ${pre-commit-check.shellHook}
            '';
          };

          # Development shells with Rust nightly using nix-lib
          devShellNightly = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain-nightly.toml;
            shellName = "HOPR Development";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              sqlite
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
              cargo-insta
              foundry-bin
              nfpm
              envsubst
            ];
            shellHook = ''
              # Fix macOS dylib loading for nightly rustfmt (rust-overlay symlink issue)
              export DYLD_LIBRARY_PATH="${nightlyToolchain}/lib:$DYLD_LIBRARY_PATH"
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
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
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
              python314
              foundry-bin
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
              python314
              foundry-bin
              (mkHoprdCandidate "")
              hopli.hopli
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
              cargo-shear
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
                pkgs-unstable.cargo-audit
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
                ${pkgs.python3}/bin/python ./scripts/find_port.py --min-port 3000 --max-port 4000 --skip 30
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
              "hoprd/rest-api-client/src/codegen/*"
              "deploy/compose/grafana/config.monitoring"
              "deploy/nfpm/nfpm.yaml"
              ".github/workflows/build-binaries.yaml"
              "docs/*"
              "hopr/builder/tests/snapshots/*"
              "hoprd/.dockerignore"
              "hoprd/rest-api/.cargo/config"
              "nix/setup-hook-darwin.sh"
              "target/*"
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
              command = "${rustfmtWrapper}/bin/rustfmt";
            };
          };

          checks = { inherit hoprd-clippy; };

          apps = {
            inherit hoprd-docker-build-and-upload;
            inherit hoprd-dev-docker-build-and-upload;
            inherit hoprd-profile-docker-build-and-upload;
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
              hoprd-localcluster
              ;
            inherit hopr-test-unit hopr-test-nightly;
            inherit docs;
            inherit pre-commit-check;
            inherit hoprd-bench;
            inherit hoprd-man;
            # binary packages
            inherit
              hoprd-x86_64-linux
              hoprd-x86_64-linux-dev
              hoprd-x86_64-linux-profile
              ;
            inherit hoprd-aarch64-linux hoprd-aarch64-linux-profile;
            # FIXME: Darwin cross-builds are currently broken.
            # Follow https://github.com/nixos/nixpkgs/pull/256590
            inherit hoprd-x86_64-darwin hoprd-x86_64-darwin-profile;
            inherit hoprd-aarch64-darwin hoprd-aarch64-darwin-profile;
            default = hoprd;
            hoprd-candidate = (mkHoprdCandidate "");
          };

          devShells.default = devShell;
          devShells.nightly = devShellNightly;
          devShells.ci = ciShell;
          devShells.test = testShell;
          devShells.citest = ciTestShell;
          devShells.docs = docsShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
    };
}
