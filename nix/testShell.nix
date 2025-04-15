{ pkgs
, extraPackages ? [ ]
, pre-commit-check
, solcDefault
, ...
}@args:
let
  mkShell = import ./mkShell.nix {};
  shellHook = ''
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ethereum/contracts/foundry.in.toml >| \
        ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi
  '' + ''
    uv sync
    unset SOURCE_DATE_EPOCH
  '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    autoPatchelf ./.venv
  '' + ''
    ${pre-commit-check.shellHook}
  '';
  packages = with pkgs; [
    uv
  ];
  shellPackages = packages ++ extraPackages;
  cleanArgs = removeAttrs args [
    "solcDefault"
    "pre-commit-check"
    "extraPackages"
  ];
in mkShell (cleanArgs // {
  inherit shellHook shellPackages;
})
