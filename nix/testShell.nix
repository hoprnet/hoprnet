{
  pkgs,
  extraPackages ? [ ],
  shellHook ? "",
  ...
}@args:
let
  mkShell = import ./mkShell.nix { };
  finalShellHook = ''
    uv sync --frozen
    unset SOURCE_DATE_EPOCH
  ''
  + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    autoPatchelf ./.venv
  ''
  + shellHook;
  packages = with pkgs; [
    uv
    python313
    foundry-bin
  ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "extraPackages"
    "shellHook"
  ];
in
mkShell (
  cleanArgs
  // {
    inherit shellPackages;
    shellHook = finalShellHook;
  }
)
