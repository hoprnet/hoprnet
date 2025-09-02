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
}:

import ../ciTestShell.nix {
  inherit pkgs config crane solcDefault hoprd hopli;
}