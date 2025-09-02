# test.nix - Test shell configuration
#
# Provides an environment configured for running tests and quality checks.
# Forms the base for other shell environments.

{ pkgs
, config
, crane
, solcDefault
, shellHook ? ""
, shellPackages ? []
, useRustNightly ? false
}:

import ../mkShell.nix {
  inherit pkgs config crane solcDefault;
  
  # Combine provided packages with test-specific tools
  shellPackages = with pkgs; [
    # Python environment for integration tests
    python313
    uv
    
    # Documentation and formatting tools
    html-tidy
    pandoc
    
    # Additional user-provided packages
  ] ++ shellPackages;
  
  # Combine shell hooks
  inherit shellHook;
  
  # Use nightly Rust if requested (for doc generation, etc.)
  inherit useRustNightly;
}