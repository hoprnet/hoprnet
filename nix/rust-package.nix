{ craneLib
, CARGO_PROFILE ? "release"
, depsSrc
, foundry-bin
, openssl
, pkg-config
, pname
, postInstall ? null
, solcDefault
, src
, version
}:
let
  sharedArgs = {
    inherit pname version CARGO_PROFILE;

    nativeBuildInputs = [ solcDefault foundry-bin pkg-config ];
    buildInputs = [ openssl ];

    CARGO_HOME = ".cargo";
    cargoExtraArgs = "--offline -p ${pname} ";
    cargoVendorDir = "vendor/cargo";
    # disable running tests automatically for now
    doCheck = false;
    # prevent nix from changing config.sub files under vendor/cargo
    dontUpdateAutotoolsGnuConfigScripts = true;
  };
in
craneLib.buildPackage (sharedArgs // {
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
