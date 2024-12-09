{ buildDocs ? false
, CARGO_PROFILE ? "release"
, cargoExtraArgs ? ""
, cargoToml
, craneLib
, curl
, depsSrc
, foundryBin
, git
, html-tidy
, isCross ? false
, lib
, libiconv
, makeSetupHook
, openssl
, pandoc
, pkg-config
, pkgs
, postInstall ? null
, rev
, runClippy ? false
, runTests ? false
, solcDefault
, src
, stdenv
}:
let
  # `hostPlatform` is the cross-compilation output platform;
  # `buildPlatform` is the platform we are compiling on
  buildPlatform = stdenv.buildPlatform;
  hostPlatform = stdenv.hostPlatform;

  # The target interpreter is used to patch the interpreter in the binary
  targetInterpreter =
    if hostPlatform.isLinux && hostPlatform.isx86_64 then "/lib64/ld-linux-x86-64.so.2"
    else if hostPlatform.isLinux && hostPlatform.isAarch64 then "/lib64/ld-linux-aarch64.so.1"
    else if hostPlatform.isLinux && hostPlatform.isArmv7 then "/lib64/ld-linux-armhf.so.3"
    else "";

  # The hook is used when building on darwin for non-darwin, where the flags
  # need to be cleaned up.
  darwinSuffixSalt = builtins.replaceStrings [ "-" "." ] [ "_" "_" ] buildPlatform.config;
  targetSuffixSalt = builtins.replaceStrings [ "-" "." ] [ "_" "_" ] hostPlatform.config;
  setupHookDarwin = makeSetupHook
    {
      name = "darwin-hopr-gcc-hook";
      substitutions = {
        inherit darwinSuffixSalt targetSuffixSalt;
      };
    } ./setup-hook-darwin.sh;

  crateInfo = craneLib.crateNameFromCargoToml { inherit cargoToml; };
  pname = crateInfo.pname;
  pnameSuffix =
    if CARGO_PROFILE == "release" then ""
    else "-${CARGO_PROFILE}";
  version = lib.strings.concatStringsSep "." (lib.lists.take 3 (builtins.splitVersion crateInfo.version));

  isDarwinForDarwin = buildPlatform.isDarwin && hostPlatform.isDarwin;
  isDarwinForNonDarwin = buildPlatform.isDarwin && !hostPlatform.isDarwin;

  darwinBuildInputs =
    if isDarwinForDarwin || isDarwinForNonDarwin then
      with pkgs.pkgsBuildHost.darwin.apple_sdk.frameworks; [
        CoreFoundation
        CoreServices
        Security
        SystemConfiguration
      ] else [ ];
  darwinNativeBuildInputs =
    if !isDarwinForDarwin && isDarwinForNonDarwin then
      [ setupHookDarwin ] else [ ];

  sharedArgsBase = {
    inherit pname pnameSuffix version CARGO_PROFILE;

    # FIXME: some dev dependencies depend on OpenSSL, would be nice to remove
    # this dependency
    nativeBuildInputs = [ solcDefault foundryBin pkg-config pkgs.pkgsBuildHost.openssl libiconv ] ++ stdenv.extraNativeBuildInputs ++ darwinNativeBuildInputs;
    buildInputs = [ openssl ] ++ stdenv.extraBuildInputs ++ darwinBuildInputs;

    CARGO_HOME = ".cargo";
    RUST_MIN_STACK = "16777216"; # 16MB required to run the tests and compilation
    cargoExtraArgs = "--offline -p ${pname} ${cargoExtraArgs}";
    # this env var is used by utoipa-swagger-ui to prevent internet access
    CARGO_FEATURE_VENDORED = "true";
    cargoVendorDir = "vendor/cargo";
    # disable running tests automatically for now
    doCheck = false;
    # prevent nix from changing config.sub files under vendor/cargo
    dontUpdateAutotoolsGnuConfigScripts = true;
    # set to the revision because during build the Git info is not available
    VERGEN_GIT_SHA = rev;
  };

  sharedArgs =
    if runTests then sharedArgsBase // {
      # exclude hopr-socks-server because it requires access to the internet
      cargoTestExtraArgs = "--workspace -F runtime-async-std -F runtime-tokio --exclude hopr-socks-server";
      doCheck = true;
    }
    else if runClippy then sharedArgsBase // { cargoClippyExtraArgs = "-- -Dwarnings"; }
    else sharedArgsBase;

  docsArgs = {
    cargoArtifacts = null;
    cargoExtraArgs = "--offline"; # overwrite the default to build all docs
    cargoDocExtraArgs = "--workspace --no-deps";
    RUSTDOCFLAGS = "--enable-index-page -Z unstable-options";
    CARGO_TARGET_DIR = "target/";
    LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.pkgsBuildHost.openssl ];
    postBuild = ''
      ${pandoc}/bin/pandoc -f markdown+hard_line_breaks -t html README.md > readme.html
      mv target/''${CARGO_BUILD_TARGET}/doc target/
      ${html-tidy}/bin/tidy -q --custom-tags yes -i target/doc/index.html > index.html || :
      sed '/<section id="main-content" class="content">/ r readme.html' index.html > target/doc/index.html
      cp index.html target/doc/index-old.html
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

  preFixup = lib.optionalString (isCross && targetInterpreter != "") ''
    for f in `find $out/bin/ -type f`; do
      echo "patching interpreter for $f to ${targetInterpreter}"
      patchelf --set-interpreter ${targetInterpreter} --output $f.patched $f
      mv $f.patched $f
    done
  '';
})
