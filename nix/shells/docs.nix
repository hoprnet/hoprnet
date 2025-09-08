# docs.nix - Documentation shell configuration
#
# Shell environment optimized for documentation generation and management.
# Includes nightly Rust for unstable doc features and additional doc tools.

{
  pkgs,
  config,
  crane,
  pre-commit-check,
  solcDefault,
}:

import ./dev.nix {
  inherit
    pkgs
    config
    crane
    pre-commit-check
    solcDefault
    ;

  # Additional packages for documentation work
  extraPackages = with pkgs; [
    html-tidy # HTML validation and formatting
    pandoc # Universal document converter
  ];

  # Use nightly Rust for documentation features
  useRustNightly = true;
}
