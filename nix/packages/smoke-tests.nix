# smoke-tests.nix - Integration smoke tests package
#
# Creates a derivation that runs smoke tests for the HOPR system.
# These tests validate basic functionality across the entire stack.

{ pkgs
, fs
, hoprdCrateInfo
, hoprdPackages
, hopliPackages
, solcDefault
}:

pkgs.stdenv.mkDerivation {
  pname = "hoprd-smoke-tests";
  version = hoprdCrateInfo.version;
  
  # Source files needed for smoke testing
  src = fs.toSource {
    root = ./../..;  # Root of the project
    fileset = fs.unions [
      # Solidity contracts for testing
      (fs.fileFilter (file: file.hasExt "sol") ./../../ethereum/contracts/src)
      # Test scripts and configurations
      ./../../tests
      ./../../scripts
      ./../../sdk/python
      # Contract configuration
      ./../../ethereum/contracts/foundry.in.toml
      ./../../ethereum/contracts/remappings.txt
    ];
  };
  
  # Dependencies for running smoke tests
  buildInputs = with pkgs; [
    uv                    # Python package manager
    foundry-bin           # Ethereum development framework
    solcDefault           # Solidity compiler
    python313             # Python runtime
    hopliPackages.hopli-dev   # HOPLI CLI for testing
    hoprdPackages.hoprd-dev   # HOPRD daemon for testing
  ];
  
  # Build phase - prepare the testing environment
  buildPhase = ''
    uv sync --frozen
    unset SOURCE_DATE_EPOCH
  '';
  
  # Check phase - run the actual smoke tests
  checkPhase = ''
    uv run --frozen -m pytest tests/
  '';
  
  # Install phase - not needed for tests, just create output
  installPhase = ''
    mkdir -p $out
    echo "Smoke tests completed successfully" > $out/test-results.txt
  '';
  
  # Enable running checks
  doCheck = true;
}