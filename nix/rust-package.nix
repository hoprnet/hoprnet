{ buildDocs ? false
, CARGO_PROFILE ? "release"
, cargoToml
, craneLib
, depsSrc
, foundryBin
, git
, html-tidy
, lib
, libiconv
, openssl
, pandoc
, pkg-config
, pkgs
, postInstall ? null
, runClippy ? false
, runTests ? false
, solcDefault
, src
, stdenv
}:
let
  crateInfo = craneLib.crateNameFromCargoToml { inherit cargoToml; };
  pname = crateInfo.pname;
  version = lib.strings.concatStringsSep "." (lib.lists.take 3 (builtins.splitVersion crateInfo.version));

  sharedArgsBase = {
    inherit pname version CARGO_PROFILE;

    # FIXME: some dev dependencies depend on OpenSSL, would be nice to remove
    # this dependency
    nativeBuildInputs = [ solcDefault foundryBin pkg-config pkgs.pkgsBuildHost.openssl libiconv ];
    buildInputs = [ ] ++ lib.optionals stdenv.isDarwin (
      with pkgs.darwin.apple_sdk.frameworks; [
        CoreServices
        CoreFoundation
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

  docsArgs = {
    cargoArtifacts = null;
    buildPhaseCargoCommand = "cargo doc --offline --no-deps";
    RUSTDOCFLAGS = "--enable-index-page -Z unstable-options";
    postBuild = ''
      ${pandoc}/bin/pandoc -f markdown+hard_line_breaks -t html README.md > readme.html
      ${html-tidy}/bin/tidy -q -i target/doc/index.html > index.html || :
      sed '/<section id="main-content" class="content">/ r readme.html' index.html > target/doc/index.html
      rm readme.html index.html
    '';
  };

  defaultArgs = {
    cargoArtifacts = craneLib.buildDepsOnly (sharedArgs // {
      src = depsSrc;
      extraDummyScript = ''
        mkdir -p $out/vendor/cargo
        cp -r --no-preserve=mode,ownership ${depsSrc}/vendor/cargo $out/vendor/
        echo "# placeholder" > $out/vendor/cargo/config.toml
      '';
    });
  };

  args = if buildDocs then sharedArgs // docsArgs else sharedArgs // defaultArgs;

  builder =
    if runTests then craneLib.cargoTest
    else if runClippy then craneLib.cargoClippy
    else if buildDocs then craneLib.cargoDoc
    else craneLib.buildPackage;
in
builder (args // {
  inherit src postInstall;

  preConfigure = ''
    # respect the amount of available cores for building
    export CARGO_BUILD_JOBS=$NIX_BUILD_CORES
    echo "# placeholder" > vendor/cargo/config.toml
    sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
      ethereum/contracts/foundry.toml.in > \
      ethereum/contracts/foundry.toml
  '';
})
