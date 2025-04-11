{ pkgs
, extraPackages ? [ ]
}@args:
let
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
in import ./mkShell.nix (args  // { extraPackages = packages ++
extraPackages;});
