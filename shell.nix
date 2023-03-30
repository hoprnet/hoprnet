{ pkgs ? import <nixpkgs> { }, ... }:
let
  linuxPkgs = with pkgs; lib.optional stdenv.isLinux (
    inotifyTools
  );
  macosPkgs = with pkgs; lib.optional stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [
      # macOS file watcher support
      CoreFoundation
      CoreServices
    ]
  );
  hoprdPkgs = with pkgs; [
    ## base
    envsubst

    ## node, minimum recommended version is v16, see README for more details
    nodejs-16_x # v16.19.1
    (yarn.override { nodejs = nodejs-16_x; }) # v3.3.0 (as per local yarn cfg)

    ## rust for core development and required utils
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    protobuf # v3.21.12
    wasm-pack # v0.10.3
    # binaryen # v111 (includes wasm-opt)
    wasm-bindgen-cli # v0.2.83

    ## python is required by node module bcrypto and integration tests
    python3 # v3.10.10
  ];
  devPkgs = with pkgs; [
    curl # v7.88.0

    # integration testing utilities
    python310Packages.pip

    # testing utilities
    jq # v1.6
    yq-go # v4.30.8

    # test Github automation
    act # 0.2.42

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
    echo "Patching foundry binaries"
    patchelf --interpreter `cat $NIX_CC/nix-support/dynamic-linker` .foundry/bin/anvil
    patchelf --interpreter `cat $NIX_CC/nix-support/dynamic-linker` .foundry/bin/cast
    patchelf --interpreter `cat $NIX_CC/nix-support/dynamic-linker` .foundry/bin/forge
    patchelf --interpreter `cat $NIX_CC/nix-support/dynamic-linker` .foundry/bin/chisel
    echo "Setup python venv"
    python -m venv .venv
    source .venv/bin/activate
    pip install -r tests/requirements.txt
  '';
}
