{
  pkgs,
  extraPackages ? [ ],
  solcDefault,
  shellHook ? "",
  ...
}@args:
let
  mkShell = import ./mkShell.nix { };
  finalShellHook = ''
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ethereum/contracts/foundry.in.toml >| \
        ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi
  ''
  + ''
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
    solcDefault
    foundry-bin
  ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "solcDefault"
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
