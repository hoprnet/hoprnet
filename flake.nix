# flake.nix - HOPR monorepo Nix flake configuration
#
# This is the main entry point for the Nix flake. It combines modular components
# from the nix/ directory to provide a complete build and development environment
# for the HOPR network software.
#
# Structure:
# - nix/inputs.nix: External dependency definitions
# - nix/lib/: Utility functions and builders
# - nix/packages/: Package definitions (hoprd, hopli)
# - nix/docker/: Docker image configurations
# - nix/shells/: Development shell environments
# - nix/apps/: Executable scripts and utilities
# - nix/checks.nix: CI/CD quality checks
# - nix/treefmt.nix: Code formatting configuration

{
  description = "HOPR Network - Privacy-preserving messaging protocol monorepo";

  # External dependencies - kept in main flake for Nix flake requirements
  #
  # INPUTS REFERENCE:
  #
  # Core Nix ecosystem dependencies:
  # - flake-utils: Provides utility functions for working with flakes across multiple systems
  # - flake-parts: Modular flake framework for better organization
  # - nixpkgs: The main Nix package repository (using release 25.05 for stability)
  #
  # Rust toolchain and build system:
  # - rust-overlay: Provides up-to-date Rust toolchains with cross-compilation support
  # - crane: Incremental Rust build system for Nix with excellent caching
  #
  # Ethereum/Solidity development tools:
  # - foundry: Ethereum development framework (pinned to specific version for reproducibility)
  # - solc: Solidity compiler packages for various versions
  #
  # Development tools and quality assurance:
  # - pre-commit: Git hooks for code quality enforcement
  # - treefmt-nix: Universal code formatter integration for Nix
  # - flake-root: Utilities for finding flake root directory
  #
  # Input optimization strategy:
  # All inputs follow nixpkgs where possible to reduce closure size and improve caching.
  # This is achieved through the "follows" directive below.
  inputs = {
    # Core Nix ecosystem dependencies
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
    
    # Rust toolchain and build system
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.21.0";
    
    # Ethereum/Solidity development tools
    foundry.url = "github:hoprnet/foundry.nix/tb/202505-add-xz";
    solc.url = "github:hellwolf/solc.nix";
    
    # Development tools and quality assurance
    pre-commit.url = "github:cachix/git-hooks.nix";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-root.url = "github:srid/flake-root";
    
    # Input dependency optimization
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
    { self
    , nixpkgs
    , flake-utils
    , flake-parts
    , rust-overlay
    , crane
    , foundry
    , solc
    , pre-commit
    , ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      # Import flake modules for additional functionality
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      
      # Per-system configuration
      # Each system gets its own set of packages, shells, etc.
      perSystem =
        { config
        , lib
        , system
        , ...
        }:
        let
          # Git revision for version tracking
          rev = toString (self.shortRev or self.dirtyShortRev);
          
          # Filesystem utilities for source filtering
          fs = lib.fileset;
          
          # System configuration
          localSystem = system;
          
          # Nixpkgs with overlays for Rust and Solidity tools
          overlays = [
            (import rust-overlay)
            foundry.overlay
            solc.overlay
          ];
          pkgs = import nixpkgs { inherit localSystem overlays; };
          
          # Platform information
          buildPlatform = pkgs.stdenv.buildPlatform;
          
          # Default Solidity compiler version
          solcDefault = solc.mkDefault pkgs pkgs.solc_0_8_19;
          
          # Crane library for Rust builds
          craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
          
          # HOPRD crate information
          hoprdCrateInfoOriginal = craneLib.crateNameFromCargoToml {
            cargoToml = ./hopr/hopr-lib/Cargo.toml;
          };
          hoprdCrateInfo = {
            pname = "hoprd";
            # Normalize version to major.minor.patch for consistent caching
            version = pkgs.lib.strings.concatStringsSep "." (
              pkgs.lib.lists.take 3 (builtins.splitVersion hoprdCrateInfoOriginal.version)
            );
          };
          
          # Import library modules
          sourcesLib = import ./nix/lib/sources.nix { inherit lib; };
          rustBuildersLib = import ./nix/lib/rust-builders.nix {
            inherit nixpkgs rust-overlay crane foundry solc localSystem;
          };
          
          # Create source trees for different build contexts
          sources = {
            main = sourcesLib.mkSrc { root = ./.; inherit fs; };
            test = sourcesLib.mkTestSrc { root = ./.; inherit fs; };
            deps = sourcesLib.mkDepsSrc { root = ./.; inherit fs; };
            anvil = sourcesLib.mkAnvilSrc { root = ./.; inherit fs; };
            pluto = sourcesLib.mkPlutoSrc { root = ./.; inherit fs; };
          };
          
          # Create all Rust builders for cross-compilation
          builders = rustBuildersLib.mkAllBuilders {};
          
          # Import package definitions
          hoprdPackages = import ./nix/packages/hoprd.nix {
            inherit lib builders sources hoprdCrateInfo rev buildPlatform;
          };
          hopliPackages = import ./nix/packages/hopli.nix {
            inherit lib builders sources rev buildPlatform;
          };
          
          # Combine all packages
          packages = hoprdPackages // hopliPackages // {
            # Additional standalone packages
            
            # Smoke tests for integration testing
            smoke-tests = pkgs.callPackage ./nix/packages/smoke-tests.nix {
              inherit fs hoprdCrateInfo hoprdPackages hopliPackages solcDefault;
            };
            
            # Pre-commit hooks check
            pre-commit-check = pkgs.callPackage ./nix/packages/pre-commit-check.nix {
              inherit pre-commit system config;
            };
            
            
            # Man pages - import as individual packages
            hoprd-man = (pkgs.callPackage ./nix/man-pages.nix {
              hoprd = hoprdPackages.hoprd-dev;
              hopli = hopliPackages.hopli-dev;
            }).hoprd-man;
            hopli-man = (pkgs.callPackage ./nix/man-pages.nix {
              hoprd = hoprdPackages.hoprd-dev;
              hopli = hopliPackages.hopli-dev;
            }).hopli-man;
          };
          
          # Import Docker configurations
          dockerBuilder = import ./nix/docker-builder.nix;
          hoprdDocker = import ./nix/docker/hoprd.nix {
            inherit pkgs dockerBuilder;
            packages = hoprdPackages;
          };
          hopliDocker = import ./nix/docker/hopli.nix {
            inherit pkgs dockerBuilder;
            packages = hopliPackages;
          };
          anvilDocker = import ./nix/docker/anvil.nix {
            inherit pkgs solcDefault;
            sources = sources;
          };
          plutoDocker = import ./nix/docker/pluto.nix {
            inherit pkgs solcDefault;
            sources = sources;
            packages = hoprdPackages // hopliPackages;
          };
          
          # Import application definitions
          dockerUploadLib = import ./nix/apps/docker-upload.nix {
            inherit pkgs flake-utils;
          };
          utilities = import ./nix/apps/utilities.nix {
            inherit pkgs system flake-utils;
          };
          
          # Import shell configurations
          shells = {
            default = import ./nix/shells/dev.nix {
              inherit pkgs config crane solcDefault;
              pre-commit-check = packages.pre-commit-check;
              extraPackages = with pkgs; [ nfpm envsubst ];
            };
            
            ci = import ./nix/shells/ci.nix {
              inherit pkgs config crane;
            };
            
            test = import ./nix/shells/test.nix {
              inherit pkgs config crane solcDefault;
            };
            
            citest = import ./nix/shells/ci-test.nix {
              inherit pkgs config crane solcDefault;
              hoprd = hoprdPackages.hoprd-candidate;
              hopli = hopliPackages.hopli-candidate;
            };
            
            citestdev = import ./nix/shells/ci-test.nix {
              inherit pkgs config crane solcDefault;
              hoprd = hoprdPackages.hoprd-dev;
              hopli = hopliPackages.hopli-dev;
            };
            
            docs = import ./nix/shells/docs.nix {
              inherit pkgs config crane solcDefault;
              pre-commit-check = packages.pre-commit-check;
            };
          };
          
          # Import checks
          checks = import ./nix/checks.nix {
            inherit pkgs solcDefault hoprdCrateInfo;
            packages = hoprdPackages // hopliPackages;
          };
          
          # Import treefmt configuration
          treefmtConfig = import ./nix/treefmt.nix {
            inherit config pkgs solcDefault;
          };
        in
        {
          # Configure treefmt
          treefmt = treefmtConfig;
          
          # Export checks for CI
          inherit checks;
          
          # Export applications
          apps = {
            # Docker upload scripts
            hoprd-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hoprdDocker.hoprd-docker;
            hoprd-dev-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hoprdDocker.hoprd-dev-docker;
            hoprd-profile-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hoprdDocker.hoprd-profile-docker;
            hopli-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hopliDocker.hopli-docker;
            hopli-dev-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hopliDocker.hopli-dev-docker;
            hopli-profile-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp hopliDocker.hopli-profile-docker;
            hopr-pluto-docker-build-and-upload = dockerUploadLib.mkDockerUploadApp plutoDocker;
            
            # Utility scripts
            inherit (utilities) update-github-labels find-port-ci;
            check = utilities.run-check;
            audit = utilities.run-audit;
          };
          
          # Export packages
          packages = packages // {
            # Docker images
            inherit (hoprdDocker) hoprd-docker hoprd-dev-docker hoprd-profile-docker;
            inherit (hopliDocker) hopli-docker hopli-dev-docker hopli-profile-docker;
            anvil-docker = anvilDocker;
            hopr-pluto = plutoDocker;
            
            # Set default package
            default = hoprdPackages.hoprd;
          };
          
          # Export development shells
          devShells = shells;
          
          # Export formatter
          formatter = config.treefmt.build.wrapper;
        };
      
      # Supported systems for building
      # Note: aarch64-linux blocked by solc support
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
    };
}