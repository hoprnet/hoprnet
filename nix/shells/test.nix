# test.nix - Test shell configuration
#
# Provides an environment configured for running tests and quality checks.
# Forms the base for other shell environments.

{
  pkgs,
  config,
  crane,
  solcDefault,
  shellHook ? "",
  shellPackages ? [ ],
  useRustNightly ? false,
  extraPackages ? [ ],
}:

let
  mkShell = import ./base.nix { };

  # Foundry setup hook for contract testing
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

  # Base packages for testing environment
  basePackages = with pkgs; [
    uv # Python package manager
    python313 # Python runtime for tests
    solcDefault # Solidity compiler
    foundry-bin # Ethereum development framework

    # Documentation and formatting tools
    html-tidy # HTML validation
    pandoc # Universal document converter
  ];

  # Combine all packages
  allShellPackages = basePackages ++ shellPackages ++ extraPackages;
in
mkShell {
  inherit
    pkgs
    config
    crane
    useRustNightly
    ;
  shellPackages = allShellPackages;
  shellHook = finalShellHook;
}
