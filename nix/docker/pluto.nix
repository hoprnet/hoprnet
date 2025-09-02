# pluto.nix - Pluto local development cluster Docker image
#
# Creates a Docker image for running a complete local HOPR development environment.
# Includes Anvil, multiple HOPRD nodes, and all necessary tooling for integration testing.

{ pkgs
, sources
, packages
, solcDefault
}:

pkgs.dockerTools.buildLayeredImage {
  name = "hopr-pluto";
  tag = "latest";
  # Note: Using "now" breaks reproducibility but makes usage easier
  created = "now";
  
  # Include comprehensive development environment
  contents = with pkgs; [
    # Core utilities
    coreutils              # Basic Unix utilities
    curl                   # HTTP client
    findutils              # File searching
    gnumake                # Build automation
    jq                     # JSON processing
    lsof                   # Network diagnostics
    openssl                # Cryptographic library
    runtimeShellPackage    # Shell interpreter
    time                   # Command timing
    tini                   # Container init system
    which                  # Command locator
    
    # Development tools
    foundry-bin            # Ethereum development framework
    solcDefault            # Solidity compiler
    python313              # Python runtime
    uv                     # Fast Python package manager
    
    # HOPR components
    packages.hoprd         # HOPR daemon
    packages.hopli         # HOPR CLI tool
    
    # Source files
    sources.pluto          # Pluto cluster configuration
  ];
  
  enableFakechroot = true;
  
  # Pre-build setup commands run during image creation
  fakeRootCommands = ''
    #!${pkgs.runtimeShell}

    # Generate the foundry.toml configuration file
    if ! grep -q "solc = \"${solcDefault}/bin/solc\"" /ethereum/contracts/foundry.toml; then
      echo "solc = \"${solcDefault}/bin/solc\""
      echo "Generating foundry.toml file!"
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        /ethereum/contracts/foundry.in.toml >| \
        /ethereum/contracts/foundry.toml
    else
      echo "foundry.toml file already exists!"
    fi

    # Rewrite remappings to use absolute paths for container environment
    sed -i \
      's|../../vendor/|/vendor/|g' \
      /ethereum/contracts/remappings.txt

    # Unlink all symbolic links (Forge compatibility)
    cp -R -L /ethereum/contracts /ethereum/contracts-unlinked
    rm -rf /ethereum/contracts
    mv /ethereum/contracts-unlinked /ethereum/contracts

    # Pre-compile contracts to speed up startup
    ${pkgs.foundry-bin}/bin/forge build --root /ethereum/contracts

    # Set up paths for development binaries
    export PATH="/target/debug/:$PATH"

    # Create temporary directories for cluster operation
    mkdir -p /tmp/hopr-localcluster
    mkdir -p /tmp/hopr-localcluster/anvil
  '';
  
  config = {
    # Set library path for OpenSSL
    Env = [
      "LD_LIBRARY_PATH=${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH"
    ];
    
    # Run the local cluster script through tini
    Cmd = [
      "/bin/tini"
      "--"
      "bash"
      "/scripts/run-local-cluster.sh"
    ];
    
    # Expose ports for various services
    ExposedPorts = {
      "8545/tcp" = { };          # Anvil Ethereum node
      "3003-3018/tcp" = { };      # HOPRD REST API ports
      "10001-10101/tcp" = { };    # HOPRD P2P ports
    };
  };
}