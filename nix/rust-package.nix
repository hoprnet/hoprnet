{
  buildDocs ? false,
  CARGO_PROFILE ? "release",
  cargoExtraArgs ? "",
  cargoToml,
  craneLib,
  depsSrc,
  foundryBin,
  html-tidy,
  isCross ? false,
  isStatic ? false,
  lib,
  libiconv,
  makeSetupHook,
  mold,
  llvmPackages,
  pandoc,
  pkg-config,
  pkgs,
  postInstall ? null,
  rev,
  runClippy ? false,
  runTests ? false,
  runBench ? false,
  solcDefault,
  src,
  stdenv,
}:
let
  # `hostPlatform` is the cross-compilation output platform
  # `buildPlatform` is the platform we are compiling on
  buildPlatform = stdenv.buildPlatform;
  hostPlatform = stdenv.hostPlatform;

  # The target interpreter is used to patch the interpreter in the binary
  targetInterpreter =
    if hostPlatform.isLinux && hostPlatform.isx86_64 then
      "/lib64/ld-linux-x86-64.so.2"
    else if hostPlatform.isLinux && hostPlatform.isAarch64 then
      "/lib64/ld-linux-aarch64.so.1"
    else
      "";

  # The hook is used when building on darwin for non-darwin, where the flags
  # need to be cleaned up.
  darwinSuffixSalt = builtins.replaceStrings [ "-" "." ] [ "_" "_" ] buildPlatform.config;
  targetSuffixSalt = builtins.replaceStrings [ "-" "." ] [ "_" "_" ] hostPlatform.config;
  setupHookDarwin = makeSetupHook {
    name = "darwin-hopr-gcc-hook";
    substitutions = { inherit darwinSuffixSalt targetSuffixSalt; };
  } ./setup-hook-darwin.sh;

  crateInfo = craneLib.crateNameFromCargoToml { inherit cargoToml; };
  pname = crateInfo.pname;
  actualCargoProfile =
    if runTests then
      "test"
    else if runClippy then
      "dev"
    else if buildDocs then
      "dev"
    else
      CARGO_PROFILE;
  pnameSuffix = if actualCargoProfile == "release" then "" else "-${actualCargoProfile}";
  pnameDeps = if actualCargoProfile == "release" then pname else "${pname}-${actualCargoProfile}";

  version = lib.strings.concatStringsSep "." (
    lib.lists.take 3 (builtins.splitVersion crateInfo.version)
  );

  isDarwinForDarwin = buildPlatform.isDarwin && hostPlatform.isDarwin;
  isDarwinForNonDarwin = buildPlatform.isDarwin && !hostPlatform.isDarwin;

  linuxNativeBuildInputs =
    if buildPlatform.isLinux then
      [
        # mold is only supported on Linux
        mold
      ]
    else
      [ ];
  darwinBuildInputs =
    if isDarwinForDarwin || isDarwinForNonDarwin then
      [
        pkgs.pkgsBuildHost.apple-sdk_15
      ]
    else
      [ ];
  darwinNativeBuildInputs =
    if !isDarwinForDarwin && isDarwinForNonDarwin then [ setupHookDarwin ] else [ ];

  buildInputs =
    if isStatic then
      with pkgs.pkgsStatic;
      [
        openssl
        cacert
      ]
    else
      with pkgs;
      [
        openssl
        cacert
      ];

  sharedArgsBase = {
    inherit pname pnameSuffix version;
    CARGO_PROFILE = actualCargoProfile;

    nativeBuildInputs = [
      llvmPackages.bintools
      solcDefault
      foundryBin
      pkg-config
      libiconv
    ]
    ++ stdenv.extraNativeBuildInputs
    ++ darwinNativeBuildInputs
    ++ linuxNativeBuildInputs;
    buildInputs = buildInputs ++ stdenv.extraBuildInputs ++ darwinBuildInputs;

    cargoExtraArgs = "-p ${pname} ${cargoExtraArgs}";
    # this env var is used by utoipa-swagger-ui to prevent internet access
    # CARGO_FEATURE_VENDORED = "true";
    strictDeps = true;
    # disable running tests automatically for now
    doCheck = false;
    # set to the revision because during build the Git info is not available
    VERGEN_GIT_SHA = rev;
  };

  sharedArgs =
    if runTests then
      sharedArgsBase
      // {
        cargoTestExtraArgs = "--workspace -F runtime-tokio";
        doCheck = true;
        LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.pkgsBuildHost.openssl ];
        RUST_BACKTRACE = "full";
      }
    else if runClippy then
      sharedArgsBase // { cargoClippyExtraArgs = "-- -Dwarnings"; }
    else if runBench then
      sharedArgsBase
      // {
        LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.pkgsBuildHost.openssl ];
      }
    else
      sharedArgsBase;

  docsArgs = {
    cargoArtifacts = null;
    cargoExtraArgs = ""; # overwrite the default to build all docs
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
    cargoArtifacts = craneLib.buildDepsOnly (
      sharedArgs
      // {
        pname = pnameDeps;
        src = depsSrc;
      }
    );
  };

  args = if buildDocs then sharedArgs // docsArgs else sharedArgs // defaultArgs;

  mkBench = import ./cargo-bench.nix {
    mkCargoDerivation = craneLib.mkCargoDerivation;
  };

  builder =
    if runTests then
      craneLib.cargoTest
    else if runClippy then
      craneLib.cargoClippy
    else if buildDocs then
      craneLib.cargoDoc
    else if runBench then
      mkBench
    else
      craneLib.buildPackage;
in
builder (
  args
  // {
    inherit src postInstall;

    preConfigure = ''
      # respect the amount of available cores for building
      export CARGO_BUILD_JOBS=$NIX_BUILD_CORES
      sed "s|# solc = .*|solc = \"${solcDefault}/bin/solc\"|g" \
        ethereum/contracts/foundry.in.toml > \
        ethereum/contracts/foundry.toml
    '';

    preFixup = lib.optionalString (isCross && targetInterpreter != "" && !isStatic) ''
      for f in `find $out/bin/ -type f`; do
        echo "patching interpreter for $f to ${targetInterpreter}"
        patchelf --set-interpreter ${targetInterpreter} --output $f.patched $f
        mv $f.patched $f
      done
    '';
  }
)
