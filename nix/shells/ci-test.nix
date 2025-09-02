# ci-test.nix - CI test shell configuration
#
# Shell environment for running integration tests in CI.
# Includes pre-built HOPRD and HOPLI binaries for faster test execution.

{ pkgs
, config
, crane
, solcDefault
, hoprd
, hopli
, extraPackages ? []
}:

import ./test.nix {
  inherit pkgs config crane solcDefault;
  
  # Include pre-built binaries for testing
  extraPackages = [
    hoprd
    hopli
  ] ++ extraPackages;
}