{
  openssl,
  pkg-config
}:
{ pname, version, src, depsSrc, craneLib, CARGO_PROFILE ? "release", postInstall ? null }:
let
  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config ];
  sharedArgs = {
    inherit pname version buildInputs nativeBuildInputs CARGO_PROFILE;
    CARGO_HOME = ".cargo";
    cargoExtraArgs = "--offline -p ${pname}";
    cargoVendorDir = "vendor/cargo";
    # disable running tests automatically for now
    doCheck = false;
    # prevent nix from changing config.sub files under vendor/cargo
    dontUpdateAutotoolsGnuConfigScripts = true;
  };
in craneLib.buildPackage sharedArgs // {
  inherit src postInstall;

  cargoArtifacts = craneLib.buildDepsOnly sharedArgs // {
    src = depsSrc;
    extraDummyScript = ''
      mkdir -p $out/vendor/cargo
      cp -r --no-preserve=mode,ownership ${depsSrc}/vendor/cargo $out/vendor/
      echo "# placeholder" > $out/vendor/cargo/config.toml
    '';
  };
}
