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
in import ./testShell.nix (args // {
  extraPackages = packages ++ extraPackages;
})
