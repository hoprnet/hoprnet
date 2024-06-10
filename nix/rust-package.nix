{ buildDocs ? false
, CARGO_PROFILE ? "release"
, cargoToml
, craneLib
, darwin
, depsSrc
, foundryBin
, git
, lib
, openssl
, pkg-config
, postInstall ? null
, solcDefault
, src
, stdenv
, runClippy ? false
, runTests ? false
}:
let
  crateInfo = craneLib.crateNameFromCargoToml { inherit cargoToml; };
  pname = crateInfo.pname;
  version = lib.strings.concatStringsSep "." (lib.lists.take 3 (builtins.splitVersion crateInfo.version));

  sharedArgsBase = {
    inherit pname version CARGO_PROFILE;

    # FIXME: some dev dependencies depend on OpenSSL, would be nice to remove
    # this dependency
    nativeBuildInputs = [ solcDefault foundryBin pkg-config openssl git ];
    buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin (
      with darwin.apple_sdk.frameworks; [
        CoreServices
        SystemConfiguration
      ]
    );

    CARGO_HOME = ".cargo";
    cargoExtraArgs = "--offline -p ${pname} ";
    cargoVendorDir = "vendor/cargo";
    # disable running tests automatically for now
    doCheck = false;
    # prevent nix from changing config.sub files under vendor/cargo
    dontUpdateAutotoolsGnuConfigScripts = true;
  };

  sharedArgs =
    if runTests then sharedArgsBase // { doCheck = true; }
    else if runClippy then sharedArgsBase // { cargoClippyExtraArgs = "-- -Dwarnings"; }
    else sharedArgsBase;

  builder =
    if runTests then craneLib.cargoTest
    else if runClippy then craneLib.cargoClippy
    else if buildDocs then craneLib.cargoDoc
    else craneLib.buildPackage;
in
builder (sharedArgs // {
  inherit src postInstall;

  cargoArtifacts = craneLib.buildDepsOnly (sharedArgs // {
    src = depsSrc;
    extraDummyScript = ''
      mkdir -p $out/vendor/cargo
      cp -r --no-preserve=mode,ownership ${depsSrc}/vendor/cargo $out/vendor/
      echo "# placeholder" > $out/vendor/cargo/config.toml
    '';
  });

  preConfigure = ''
    # respect the amount of available cores for building
    export CARGO_BUILD_JOBS=$NIX_BUILD_CORES
    echo "# placeholder" > vendor/cargo/config.toml
    sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
      ethereum/contracts/foundry.toml.in > \
      ethereum/contracts/foundry.toml
  '';
})
