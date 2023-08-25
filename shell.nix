{ pkgs ? import <nixpkgs> { }
, pkgs-dev
, ...
}:
let
  linuxPkgs = with pkgs; lib.optional stdenv.isLinux (
    inotifyTools
  );
  macosPkgs = with pkgs; lib.optional stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [
      SystemConfiguration
      # macOS file watcher support
      CoreFoundation
      CoreServices
    ]
  );
  hoprdPkgs = with pkgs; [
    ## base
    envsubst

    ## node, minimum recommended version is v18, see README for more details
    nodejs-18_x # v18.16.1
    (yarn.override { nodejs = nodejs-18_x; }) # v3.6.0 (as per local yarn cfg)

    ## rust for core development and required utils
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    protobuf # v3.21.12
    pkgs-dev.wasm-pack # v0.11.1
    pkgs-dev.binaryen # v113 (includes wasm-opt)
    wasm-bindgen-cli # v0.2.83
    pkg-config

    ## python is required by node module bcrypto and integration tests
    python39 # v3.10.12
  ];
  devPkgs = with pkgs; [
    patchelf

    curl # v7.88.0

    # integration testing utilities
    python39Packages.pip

    # testing utilities
    jq # v1.6
    yq-go # v4.30.8

    # test Github automation
    act # 0.2.42

    # test coverage generation
    lcov

    # custom pkg groups
    macosPkgs
    linuxPkgs
  ];
in
with pkgs;
mkShell {
  buildInputs = hoprdPkgs ++ devPkgs;
  shellHook = ''
    echo "Installing dependencies"
    make deps

    echo "Setting up python virtual environment"
    python -m venv .venv
    source .venv/bin/activate
    pip install -r tests/requirements.txt
    deactivate

    echo "Patching additional binaries"
    patchelf --interpreter `cat $NIX_CC/nix-support/dynamic-linker` .venv/bin/ruff
  '';
}
