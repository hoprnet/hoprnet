{ }:

{
  config,
  pkgs,
  crane,
  shellHook ? "",
  shellPackages ? [ ],
  useRustNightly ? false,
}:
let
  buildPlatform = pkgs.stdenv.buildPlatform;
  cargoTarget =
    if buildPlatform.config == "arm64-apple-darwin" then
      "aarch64-apple-darwin"
    else
      buildPlatform.config;
  rustToolchain =
    if useRustNightly then
      pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)
    else
      (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml).override {
        targets = [ cargoTarget ];
      };
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  linuxMinimumPackages = pkgs.lib.optionals pkgs.stdenv.isLinux (
    with pkgs;
    [
      # mold is only supported on Linux
      mold
      autoPatchelfHook
    ]
  );
  minimumPackages =
    with pkgs;
    [
      bash
      coreutils
      curl
      findutils
      gnumake
      gnuplot
      jq
      just
      llvmPackages.bintools
      lsof
      openssl
      patchelf
      pkg-config
      time
      which
      yq-go
      help2man

      ## formatting
      config.treefmt.build.wrapper
    ]
    ++ (pkgs.lib.attrValues config.treefmt.build.programs)
    ++ linuxMinimumPackages;
  packages = minimumPackages ++ shellPackages;

  # mold is only supported on Linux, so falling back to lld on Darwin
  linker = if buildPlatform.isDarwin then "lld" else "mold";
in
craneLib.devShell {
  inherit shellHook packages;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
    [
      pkgs.pkgsBuildHost.openssl
      pkgs.pkgsBuildHost.curl
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.pkgsBuildHost.libgcc.lib ]
  );
  CARGO_BUILD_RUSTFLAGS = "-C link-arg=-fuse-ld=${linker}";
  HOPR_INTERNAL_TRANSPORT_ACCEPT_PRIVATE_NETWORK_IP_ADDRESSES = "true"; # Allow local private IPs in dev shells
}
