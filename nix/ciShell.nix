{
  pkgs,
  extraPackages ? [ ],
  ...
}@args:
let
  mkShell = import ./mkShell.nix { };
  packages = with pkgs; [
    act
    gh
    google-cloud-sdk
    graphviz
    lcov
    skopeo
    swagger-codegen3
    vacuum-go
    zizmor
    nfpm
    envsubst
    gnupg
    perl

    # testing utilities
    cargo-audit

    # docker image inspection and handling
    dive

    uv
    python313
  ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "extraPackages"
  ];
in
mkShell (
  cleanArgs
  // {
    inherit shellPackages;
  }
)
