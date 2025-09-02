# anvil.nix - Anvil local Ethereum node Docker image
#
# Creates a Docker image for running a local Ethereum development node using Anvil.
# This is used for local testing and development of HOPR smart contracts.

{
  pkgs,
  sources,
  solcDefault,
}:

pkgs.dockerTools.buildLayeredImage {
  name = "hopr-anvil";
  tag = "latest";
  # Note: Using "now" breaks reproducibility but makes usage easier
  created = "now";

  # Include all necessary tools and dependencies
  contents = with pkgs; [
    sources.anvil # Anvil source files
    coreutils # Basic Unix utilities
    curl # HTTP client for API calls
    findutils # File searching utilities
    foundry-bin # Foundry toolkit (includes Anvil)
    gnumake # Build automation
    jq # JSON processor
    lsof # Network diagnostics
    runtimeShellPackage # Shell interpreter
    solcDefault # Solidity compiler
    time # Command timing
    tini # Minimal init system for containers
    which # Command locator
  ];

  enableFakechroot = true;

  # Pre-build setup commands run during image creation
  fakeRootCommands = ''
    #!${pkgs.runtimeShell}

    # Generate the foundry.toml configuration file with correct solc path
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        /ethereum/contracts/foundry.in.toml >| \
        /ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi

    # Rewrite remappings to use absolute paths
    # This fixes solc compilation checks in the container environment
    sed -i \
      's|../../vendor/|/vendor/|g' \
      /ethereum/contracts/remappings.txt

    # Unlink all symbolic links in the contracts directory
    # Forge doesn't work well with symlinks, so we create real copies
    cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
    rm -rf /ethereum/contracts
    mv /ethereum/contracts-unlinked /ethereum/contracts

    # Pre-compile contracts to speed up container startup
    ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts
  '';

  config = {
    # Use tini as init to properly handle signals and reap zombies
    Cmd = [
      "/bin/tini"
      "--"
      "bash"
      "/scripts/run-local-anvil.sh"
    ];

    # Expose the standard Ethereum JSON-RPC port
    ExposedPorts = {
      "8545/tcp" = { };
    };
  };
}
