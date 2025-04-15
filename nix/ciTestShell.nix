{ pkgs
, extraPackages ? [ ]
, pre-commit-check
, solcDefault
, hoprd
, hopli
, ...
}@args:
let
  packages = with pkgs; [
    hoprd
    hopli
  ];
  cleanArgs = removeAttrs args [
    "hoprd"
    "hopli"
  ];
in
import ./testShell.nix (cleanArgs // {
  extraPackages = packages ++ extraPackages;
})
