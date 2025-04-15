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
  minimumPackages = with pkgs; [
    bash
    coreutils
    curl
    findutils
    gnumake
    jq
    lsof
    openssl
    patchelf
    pkg-config
    time
    which
    yq-go

    ## formatting
    config.treefmt.build.wrapper
  ] ++
  (lib.attrValues config.treefmt.build.programs) ++
  lib.optionals stdenv.isLinux [ autoPatchelfHook ] ++ extraPackages;
  shellHook = ''
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ethereum/contracts/foundry.in.toml >| \
        ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi
  '' + ''
    uv sync
    unset SOURCE_DATE_EPOCH
  '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    autoPatchelf ./.venv
  '' + ''
    ${pre-commit-check.shellHook}
  '';
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ([ pkgs.pkgsBuildHost.openssl ] ++
    pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.pkgsBuildHost.libgcc.lib ]);
  }
  }
