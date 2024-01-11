{ pkgs ? import <nixpkgs> { }
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

    ## rust for core development and required utils
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    protobuf # v3.21.12
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
  nativeBuildInputs = [ openssl.dev pkg-config ];
  buildInputs = hoprdPkgs ++ devPkgs;
  shellHook = ''
    echo "Install cargo utils (dependency pruning...)"
    cargo install cargo-machete

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
