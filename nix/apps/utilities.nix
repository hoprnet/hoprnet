# utilities.nix - Utility applications and scripts
#
# Provides various utility scripts for development, testing, and maintenance.

{
  pkgs,
  system,
  flake-utils,
}:

{
  # Run all or specific checks for the project
  # Without arguments: runs all checks
  # With argument: runs specific check by name
  check = flake-utils.lib.mkApp {
    drv = pkgs.writeShellScriptBin "check" ''
      set -e
      check=$1
      if [ -z "$check" ]; then
        # Run all checks by listing them from flake output
        nix flake show --json 2>/dev/null | \
          jq -r '.checks."${system}" | to_entries | .[].key | @sh' | \
          xargs -I '{}' sh -c 'nix build ".#checks.${system}.$1"' -- {}
      else
        # Run specific check
        nix build ".#checks.${system}.$check"
      fi
    '';
  };

  # Run cargo audit for security vulnerability checking
  audit = flake-utils.lib.mkApp {
    drv = pkgs.writeShellApplication {
      name = "audit";
      runtimeInputs = with pkgs; [
        cargo
        cargo-audit
      ];
      text = ''
        cargo audit
      '';
    };
  };

  # Find an available port for CI testing
  # Used to avoid port conflicts in parallel CI runs
  find-port-ci = flake-utils.lib.mkApp {
    drv = pkgs.writeShellApplication {
      name = "find-port";
      text = ''
        ${pkgs.python3}/bin/python ${./../../tests/find_port.py} \
          --min-port 3000 \
          --max-port 4000 \
          --skip 30
      '';
    };
  };

  # Update GitHub labels configuration based on crate structure
  # Automatically generates labels for each crate in the monorepo
  update-github-labels = flake-utils.lib.mkApp {
    drv = pkgs.writeShellScriptBin "update-github-labels" ''
      set -eu

      # Remove existing crate entries to handle removed crates
      yq 'with_entries(select(.key != "crate:*"))' \
        .github/labeler.yml > labeler.yml.new

      # Add new crate entries for all known crates
      for f in `find . -mindepth 2 -name "Cargo.toml" -type f -printf '%P\n'`; do
        env \
          name="crate:`yq '.package.name' $f`" \
          dir="`dirname $f`/**" \
          yq -n '.[strenv(name)][0]."changed-files"[0]."any-glob-to-any-file" = env(dir)' \
          >> labeler.yml.new
      done

      mv labeler.yml.new .github/labeler.yml
    '';
  };
}
