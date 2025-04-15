{ pkgs
, extraPackages ? [ ]
, ...
}@args:
let
  mkShell = import ./mkShell.nix {};
  packages = with pkgs; [
    act
    dive
    gh
    google-cloud-sdk
    graphviz
    lcov
    skopeo
    swagger-codegen3
    vacuum-go
    zizmor

    # testing utilities
    cargo-audit

    # docker image inspection and handling
    dive
  ];
in mkShell (args  // {
  extraPackages = packages ++ extraPackages;
})
