{
  description = "hoprnet monorepo";

  inputs.flake-parts.url = github:hercules-ci/flake-parts;
  inputs.nixpkgs.url = github:NixOS/nixpkgs/master;
  inputs.rust-overlay.url = github:oxalica/rust-overlay;

  inputs.rust-overlay.inputs = {
    nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-parts, rust-overlay, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      perSystem = { config, self', inputs', system, ... }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          devShells.default = import ./shell.nix {
            inherit pkgs;
          };
        };
      systems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin"];
      flake = {
        overlays = [
          rust-overlay.overlays
        ];
      };
    };
}
