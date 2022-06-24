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
in
with pkgs;
mkShell {
  buildInputs = [
    ## base
    envsubst

    ## node, minimum recommended version is v16, see README for more details
    nodejs-16_x # v16.5.0
    (yarn.override { nodejs = nodejs-16_x; }) # v1.22.10

    ## rust for core development
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)

    ## python is required by node module bcrypto
    python3

    # test Github automation
    act

    # testing utilities
    websocat
    jq
    vagrant
    shellcheck

    # devops tooling
    google-cloud-sdk

    # custom pkg groups
    macosPkgs
    linuxPkgs
  ];
}
