{
  description = "hoprnet monorepo";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-22.11;
  inputs.rust-overlay.url = github:oxalica/rust-overlay;

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "hoprnet";
      shell = ./shell.nix;
      systems = [ "x86_64-linux" "aarch64-darwin" ];
      preOverlays = [
        rust-overlay.overlays.default
      ];
    };
}
