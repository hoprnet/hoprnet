# ci.nix - CI/CD shell configuration
#
# Minimal shell environment for continuous integration pipelines.
# Contains only essential tools to keep CI runs fast and reproducible.

{
  pkgs,
  config,
  crane,
  extraPackages ? [ ],
}:

let
  mkShell = import ./base.nix { };

  # CI-specific packages
  packages = with pkgs; [
    act # GitHub Actions local runner
    gh # GitHub CLI
    google-cloud-sdk # Google Cloud tools
    graphviz # Graph visualization
    lcov # Code coverage
    skopeo # Container image tools
    swagger-codegen3 # API code generation
    vacuum-go # OpenAPI linting
    zizmor # GitHub Actions security analysis
    nfpm # Package manager
    envsubst # Environment variable substitution
    gnupg # GPG encryption
    perl # Perl interpreter

    # Testing utilities
    cargo-audit # Rust security auditing

    # Docker image inspection and handling
    dive # Docker layer analysis

    # Python environment
    uv # Python package manager
    python313 # Python runtime
  ];

  shellPackages = packages ++ extraPackages;
in
mkShell {
  inherit
    pkgs
    config
    crane
    shellPackages
    ;
}
