# Definition of Nix packages compatible with flakes and traditional workflow.
let
  lockFile = import ./flake-lock.nix { src = ./..; };
in
{ localSystem ? builtins.currentSystem
, crossSystem ? null
, src ? lockFile.nixpkgs
, config ? { }
, overlays ? [ ]
}:
let
  # Use the exact nixpkgs revision as the one used by nixpkgs-cross-overlay itself.
  nixpkgs = "${lockFile.nixpkgs-cross-overlay}/utils/nixpkgs.nix";

  pkgs = import nixpkgs {
    inherit localSystem crossSystem overlays;
  };
in
pkgs
