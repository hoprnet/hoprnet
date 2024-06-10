{ config, pkgs, crane, pre-commit-check, solcDefault, extraPackages ? [ ] }:
let
  cargoTarget = pkgs.stdenv.buildPlatform.config;
  rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override { targets = [ cargoTarget ]; };
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
in
craneLib.devShell {
  packages = with pkgs; [
    openssl
    pkg-config
    foundry-bin
    solcDefault

    # testing utilities
    jq
    yq-go
    curl
    bash
    gnumake
    which

    # github integration
    gh

    # test Github automation
    act

    # documentation utilities
    pandoc
    swagger-codegen3

    # docker image inspection and handling
    dive
    skopeo

    # test coverage generation
    lcov

    ## python is required by integration tests
    python39
    python39Packages.venvShellHook

    ## formatting
    config.treefmt.build.wrapper
  ] ++
  (lib.attrValues config.treefmt.build.programs) ++
  lib.optionals stdenv.isLinux [ autoPatchelfHook ] ++ extraPackages;
  venvDir = "./.venv";
  postVenvCreation = ''
    unset SOURCE_DATE_EPOCH
    pip install -U pip setuptools wheel
    pip install -r tests/requirements.txt
  '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    autoPatchelf ./.venv
  '';
  preShellHook = ''
    sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
      ethereum/contracts/foundry.toml.in > \
      ethereum/contracts/foundry.toml
  '';
  postShellHook = ''
    ${pre-commit-check.shellHook}
  '';
}
