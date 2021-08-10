{
  description = "hoprnet monorepo";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixpkgs-unstable;

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "hoprnet";
      shell = ./shell.nix;
      systems = [ "x86_64-linux" "aarch64-darwin" ];
    };
}
