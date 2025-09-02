# checks.nix - CI/CD quality checks
#
# Defines automated checks that run in CI to ensure code quality.
# These checks can also be run locally for pre-push validation.

{ pkgs
, packages
, solcDefault
, hoprdCrateInfo
}:

{
  # Rust linting checks
  hoprd-clippy = packages.hoprd-clippy;
  hopli-clippy = packages.hopli-clippy;

  # Check that generated Rust bindings are up-to-date
  # This ensures contract changes are reflected in the Rust code
  check-bindings = pkgs.stdenv.mkDerivation {
    pname = "check-bindings";
    version = hoprdCrateInfo.version;
    
    src = ../.;
    
    buildInputs = with pkgs; [
      diffutils         # For comparing files
      foundry-bin       # Forge for contract compilation
      solcDefault       # Solidity compiler
      just              # Task runner
    ];
    
    preConfigure = ''
      # Generate foundry.toml with correct solc path
      mkdir -p ethereum/contracts
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ${../ethereum/contracts/foundry.in.toml} > ./ethereum/contracts/foundry.toml
    '';
    
    buildPhase = ''
      # Generate new bindings
      just generate-bindings
    '';
    
    checkPhase = ''
      echo "Checking if generated bindings introduced changes..."
      
      # If reference directory exists, bindings are outdated
      if [ -d "ethereum/bindings/src/reference" ]; then
        echo "Generated bindings are outdated."
        echo "Please run the binding generation and commit the changes."
        exit 1
      fi
      
      echo "Bindings are up to date."
    '';
    
    # Minimal install phase (we only care about the check)
    installPhase = "mkdir -p $out";
    doCheck = true;
  };
}