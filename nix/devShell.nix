{
  pkgs,
  extraPackages ? [ ],
  pre-commit-check,
  solcDefault,
  ...
}@args:
let
  shellHook = ''
    ${pre-commit-check.shellHook}
  '';
  packages = with pkgs; [ cargo-llvm-cov ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "pre-commit-check"
  ];
in
<<<<<<< HEAD
import ./testShell.nix (cleanArgs // {
  inherit shellHook;
  extraPackages = shellPackages;
})
||||||| 82381cf104
import ./testShell.nix (cleanArgs // {
  inherit shellHook shellPackages;
})
=======
import ./testShell.nix (
  cleanArgs
  // {
    inherit shellHook shellPackages;
  }
)
>>>>>>> origin/master
