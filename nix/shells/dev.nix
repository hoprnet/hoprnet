# dev.nix - Development shell configuration
#
# Provides a comprehensive development environment with all necessary tools
# for working on the HOPR monorepo. Includes pre-commit hooks and testing tools.

{ pkgs
, config
, crane
, pre-commit-check
, solcDefault
, extraPackages ? []
, useRustNightly ? false
}:

let
  # Import the base test shell and extend it with development tools
  baseShell = import ../testShell.nix {
    inherit pkgs config crane solcDefault useRustNightly;
    
    # Additional packages for development
    shellPackages = with pkgs; [
      sqlite                # Database for local testing
    ] ++ extraPackages;
    
    # Set up pre-commit hooks on shell entry
    shellHook = ''
      ${pre-commit-check.shellHook}
    '';
  };
in
baseShell