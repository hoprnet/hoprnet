{
  description = "hopr-lib repository";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/master";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.23.0";
    nix-lib.url = "github:hoprnet/nix-lib/v1.1.0";
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

          # Use nix-lib's source filtering for better rebuild performance
          depsSrc = nixLib.mkDepsSrc {
            root = ./.;
            inherit fs;
          };
          src = nixLib.mkSrc {
            root = ./.;
            inherit fs;
          };
          testSrc = nixLib.mkTestSrc {
            root = ./.;
            inherit fs;
            extraFiles = [
              ./hopr/reference/tests
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

          # Coverage builder with llvm-tools for code coverage instrumentation
          rust-builder-local-coverage = builders.localCoverage;

          # Nightly builder for docs and specific features
          # Uses a pinned nightly version to avoid ICE bugs in latest nightly
          rust-builder-local-nightly = nixLib.mkRustBuilder {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain-nightly.toml;
          };

          libraryBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "";
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          ticketInspectorBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "-p hopr-ticket-manager --bin ticket-inspector -F redb,serde,cli";
            cargoToml = ./logic/ticket-manager/Cargo.toml;
          };

          # Shared preBuild hook to fix stale sandbox paths in cached utoipa-swagger-ui build script outputs
          fixUtoipaEmbedPaths =
            drv:
            drv.overrideAttrs (old: {
              preBuild = ''
                find target -name 'embed.rs' -path '*/utoipa-swagger-ui*/out/*' \
                  -exec sed -i "s|/nix/var/nix/builds/[^/]*/source|$(pwd)|g" {} \;
              ''
              + (old.preBuild or "");
            });

          hoprPackages = {
            # ticket-inspector: diagnostic CLI for inspecting the tickets database
            binary-ticket-inspector = rust-builder-local.callPackage nixLib.mkRustPackage ticketInspectorBuildArgs;
            binary-ticket-inspector-x86_64-linux = rust-builder-x86_64-linux.callPackage nixLib.mkRustPackage ticketInspectorBuildArgs;
            binary-ticket-inspector-aarch64-linux = rust-builder-aarch64-linux.callPackage nixLib.mkRustPackage ticketInspectorBuildArgs;
            test-unit =
              (fixUtoipaEmbedPaths (
                rust-builder-local.callPackage nixLib.mkRustPackage (
                  libraryBuildArgs
                  // {
                    src = testSrc;
                    cargoExtraArgs = "-F allocator-jemalloc";
                    runTests = true;
                    prependPackageName = false;
                    cargoTestExtraArgs = "--lib";
                    extraNativeBuildInputs = [ pkgs.cargo-nextest ];
                  }
                )
              )).overrideAttrs
                (_: {
                  checkPhase = ''
                    runHook preCheck
                    cargo nextest run ''${CARGO_PROFILE:+--cargo-profile $CARGO_PROFILE} -F allocator-jemalloc --lib
                    runHook postCheck
                  '';
                });

            test-integration =
              (fixUtoipaEmbedPaths (
                rust-builder-local.callPackage nixLib.mkRustPackage (
                  libraryBuildArgs
                  // {
                    src = testSrc;
                    cargoExtraArgs = "-F allocator-jemalloc";
                    runTests = true;
                    prependPackageName = false;
                    cargoTestExtraArgs = "--test '*' -- --test-threads=1";
                    extraNativeBuildInputs = [ pkgs.cargo-nextest ];
                  }
                )
              )).overrideAttrs
                (_: {
                  checkPhase = ''
                    runHook preCheck
                    cargo nextest run ''${CARGO_PROFILE:+--cargo-profile $CARGO_PROFILE} -F allocator-jemalloc --test '*' -j 1
                    runHook postCheck
                  '';
                });

            test-nightly =
              (fixUtoipaEmbedPaths (
                rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
                  libraryBuildArgs
                  // {
                    src = testSrc;
                    cargoExtraArgs = "-Z panic-abort-tests -F allocator-jemalloc";
                    runTests = true;
                    prependPackageName = false;
                    cargoTestExtraArgs = "--lib";
                    extraNativeBuildInputs = [ pkgs.cargo-nextest ];
                  }
                )
              )).overrideAttrs
                (_: {
                  checkPhase = ''
                    runHook preCheck
                    cargo nextest run ''${CARGO_PROFILE:+--cargo-profile $CARGO_PROFILE} -Z panic-abort-tests -F allocator-jemalloc --lib
                    runHook postCheck
                  '';
                });

            # Code coverage (outputs LCOV report)
            coverage-unit =
              (fixUtoipaEmbedPaths (
                rust-builder-local-coverage.callPackage nixLib.mkRustPackage (
                  libraryBuildArgs
                  // {
                    src = testSrc;
                    cargoExtraArgs = "-F allocator-jemalloc";
                    runCoverage = true;
                    prependPackageName = false;
                    cargoLlvmCovExtraArgs = "--lcov --output-path $out --lib";
                    extraNativeBuildInputs = [ pkgs.cargo-nextest ];
                  }
                )
              )).overrideAttrs
                (_: {
                  buildPhase = ''
                    runHook preBuild
                    cargo llvm-cov nextest --lcov --output-path $out --lib \
                      ''${CARGO_PROFILE:+--cargo-profile $CARGO_PROFILE} \
                      --workspace -F allocator-jemalloc
                    runHook postBuild
                  '';
                });

            workspace-clippy = rust-builder-local.callPackage nixLib.mkRustPackage (
              libraryBuildArgs // { runClippy = true; }
            );

            # Build all workspace benchmarks without running them; copies binaries to $out/bin
            bench-build =
              (rust-builder-local.callPackage nixLib.mkRustPackage (libraryBuildArgs // { buildBench = true; }))
              .overrideAttrs
                (old: {
                  postInstall = (if old ? postInstall && old.postInstall != null then old.postInstall else "") + ''
                    mkdir -p "$out/bin"
                    find target -maxdepth 4 -path '*/deps/*' -type f -name "*_bench-*" \
                      -not -name "*.*" \
                      -exec cp {} "$out/bin/" \;
                  '';
                });
          };

          docs = rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
            libraryBuildArgs // { buildDocs = true; }
          );

          # pre-commit in nixpkgs bundles heavyweight test-only dependencies
          # (dotnet-sdk, nodejs, go, coursier, …) into nativeBuildInputs via
          # its preCheck string interpolation, even though doCheck is already
          # false on Darwin. Filter them out so `direnv allow` / `nix develop`
          # doesn't have to build dotnet from source.
          pre-commit-lightweight = pkgs.pre-commit.overridePythonAttrs {
            nativeCheckInputs = [ ];
            doCheck = false;
            doInstallCheck = false;
            dontUsePytestCheck = true;
            preCheck = "";
            postCheck = "";
          };

          pre-commit-check = pre-commit.lib.${system}.run {
            src = ./.;
            package = pre-commit-lightweight;
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
              sync-copilot-instructions = {
                enable = true;
                name = "Sync .claude/ instructions to Copilot files";
                entry = "bash .github/scripts/sync-copilot-instructions.sh";
                files = "(\\.claude/INSTRUCTIONS\\.md|\\.claude/rust\\.md)";
                language = "system";
                pass_filenames = false;
              };
              generate-metrics-docs = {
                enable = true;
                name = "METRICS.md must stay in sync with code";
                entry = "bash .github/scripts/generate-metrics-docs.sh --fix";
                files = "(METRICS\\.md|\\.rs)$";
                pass_filenames = false;
                language = "system";
              };
              check-bench-names = {
                enable = true;
                name = "Benchmark names must end with _bench";
                entry = "bash .github/scripts/check-bench-names.sh";
                files = "Cargo\\.toml$";
                pass_filenames = true;
                language = "system";
              };
            };
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
              cargo-nextest
              foundry-bin
              nfpm
              envsubst
              uv
              graphviz
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
          coverageShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "HOPR Coverage";
            withLlvmTools = true;
            extraPackages = with pkgs; [
              sqlite
              cargo-nextest
            ];
          };

          run-check = nixLib.mkCheckApp { inherit system; };
          run-audit = nixLib.mkAuditApp {
            rustToolchainFile = ./rust-toolchain.toml;
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
              ".github/workflows/build-binaries.yaml"
              "docs/*"
              "hopr/reference/tests/snapshots/*"
              "nix/setup-hook-darwin.sh"
              "target/*"
            ];

            programs.shfmt.enable = true;

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

          checks = { inherit (hoprPackages) workspace-clippy; };

          apps = {
            inherit update-github-labels find-port-ci;
            check = run-check;
            audit = run-audit;
            bench-run = {
              type = "app";
              program = toString (
                pkgs.writeShellScript "bench-run" ''
                  set -euo pipefail
                  shopt -s nullglob
                  bins=(result/bin/*)
                  if [ ''${#bins[@]} -eq 0 ]; then
                    nix build -L .#bench-build
                    bins=(result/bin/*)
                  fi
                  if [ ''${#bins[@]} -eq 0 ]; then
                    echo "No benchmark binaries found under result/bin" >&2
                    exit 1
                  fi
                  for bin in "''${bins[@]}"; do
                    "$bin" --bench
                  done
                ''
              );
              meta.description = "Run all benchmarks";
            };
          };

          packages = hoprPackages // {
            inherit docs;
            inherit pre-commit-check;
            default = hoprPackages.binary-ticket-inspector;
          };

          devShells.default = devShell;
          devShells.nightly = devShellNightly;
          devShells.ci = ciShell;
          devShells.test = testShell;
          devShells.citest = ciTestShell;
          devShells.docs = docsShell;
          devShells.coverage = coverageShell;

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
