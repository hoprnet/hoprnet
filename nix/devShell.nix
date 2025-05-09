{ pkgs
, extraPackages ? [ ]
, pre-commit-check
, solcDefault
, ...
}@args:
let
  shellHook = ''
    ${pre-commit-check.shellHook}
  '';
  packages = [ ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "pre-commit-check"
  ];
in
import ./testShell.nix (cleanArgs // {
  inherit shellHook shellPackages;
})
