# inputs.nix - Flake input documentation
# 
# This file documents all external dependencies used by the HOPR monorepo flake.
# Note: Due to Nix flake restrictions, the actual inputs must be defined in flake.nix
# This file serves as documentation and reference for the inputs used.
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
# This is achieved through the "follows" directive in flake.nix.

{}