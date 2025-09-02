# ci.nix - CI/CD shell configuration
#
# Minimal shell environment for continuous integration pipelines.
# Contains only essential tools to keep CI runs fast and reproducible.

{ pkgs
, config
, crane
}:

import ../ciShell.nix {
  inherit pkgs config crane;
}