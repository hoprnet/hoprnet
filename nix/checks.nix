# checks.nix - CI/CD quality checks
#
# Defines automated checks that run in CI to ensure code quality.
# These checks can also be run locally for pre-push validation.

{
  pkgs,
  packages,
  solcDefault,
  hoprdCrateInfo,
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
      diffutils # For comparing files
      foundry-bin # Forge for contract compilation
      solcDefault # Solidity compiler
      just # Task runner
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

      # Create backup of current committed bindings
      cp -r ethereum/bindings/src/codegen ethereum/bindings/src/codegen.backup

      # Regenerate bindings
      just generate-bindings

      # Compare the regenerated bindings against the committed version
      if ! diff -ru ethereum/bindings/src/codegen.backup ethereum/bindings/src/codegen; then
        echo ""
        echo "ERROR: Generated bindings differ from committed bindings."
        echo "Please regenerate bindings and commit the changes:"
        echo "  just generate-bindings"
        echo "  git add ethereum/bindings/src/codegen"
        echo "  git commit -m 'feat(bindings): Update generated bindings'"
        exit 1
      fi

      echo "Bindings are up to date."
    '';

    # Minimal install phase (we only care about the check)
    installPhase = "mkdir -p $out";
    doCheck = true;
  };
}
