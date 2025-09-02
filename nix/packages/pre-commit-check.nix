# pre-commit-check.nix - Pre-commit hooks configuration package
#
# Defines the pre-commit hooks that run automatically before each commit
# to ensure code quality, formatting, and basic validation.

{ pre-commit
, system
, config
, pkgs
}:

pre-commit.lib.${system}.run {
  src = ./../..;  # Root of the project
  
  # Configure the pre-commit hooks to run
  hooks = {
    # Use treefmt for code formatting (disabled by default, enabled via package)
    treefmt.enable = false;
    treefmt.package = config.treefmt.build.wrapper;
    
    # Shell script validation
    check-executables-have-shebangs.enable = true;
    check-shebang-scripts-are-executable.enable = true;
    
    # File system checks
    check-case-conflicts.enable = true;
    check-symlinks.enable = true;
    check-merge-conflicts.enable = true;
    check-added-large-files.enable = true;
    
    # Commit message formatting
    commitizen.enable = true;
    
    # Custom immutable files check (disabled by default)
    immutable-files = {
      enable = false;
      name = "Immutable files - the files should not change";
      entry = "bash .github/scripts/immutable-files-check.sh";
      files = "";
      language = "system";
    };
  };
  
  # Tools available to the pre-commit environment
  tools = pkgs;
  
  # Exclude certain paths from pre-commit checks
  excludes = [
    "vendor/"                           # Third-party code
    "ethereum/contracts/"               # Generated/external contracts
    "ethereum/bindings/src/codegen"     # Generated bindings
    ".gcloudignore"                     # Cloud configuration
  ];
}