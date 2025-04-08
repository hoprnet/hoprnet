{ config
, pkgs
, crane
, pre-commit-check
, solcDefault
, extraPackages ? [ ]
, useRustNightly ? false
}:
let
  cargoTarget = pkgs.stdenv.buildPlatform.config;
  rustToolchain =
    if useRustNightly
    then pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)
    else
      (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
        ../rust-toolchain.toml).override { targets = [ cargoTarget ]; };
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
in
craneLib.devShell {
  packages = with pkgs; [
    openssl
    pkg-config
    patchelf
    foundry-bin
    solcDefault

    # testing utilities
    jq
    yq-go
    curl
    bash
    cargo-audit

    # anvil
    gnumake
    lsof
    coreutils
    which
    findutils
    time

    # docs utilities
    graphviz

    # github integration
    gh

    # Github automation
    act
    zizmor

    # documentation utilities
    swagger-codegen3
    vacuum-go

    # docker image inspection and handling
    dive
    skopeo

    # test coverage generation
    lcov

    ## python is required by integration tests
    python313
    python313Packages.venvShellHook
    uv

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
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ethereum/contracts/foundry.in.toml >| \
        ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi
  '';
  postShellHook = ''
    ${pre-commit-check.shellHook}
  '';
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ([ pkgs.pkgsBuildHost.openssl ] ++
    pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.pkgsBuildHost.libgcc.lib ]);
}
