let
  sources = import ./nix/sources.nix;
  stable = import sources.stable { };
  unstable = import sources.unstable { };

  linuxPkgs = with stable; stdenv.lib.optional stdenv.isLinux (
    inotifyTools
  );
  macosPkgs = with stable; stdenv.lib.optional stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [
      # macOS file watcher support
      CoreFoundation
      CoreServices
    ]
  );
in
with stable;
mkShell {
  buildInputs = [
    ## base
    git
    lsof
    unstable.niv
    shellcheck

    ## node, minimum recommended version is v14, see README for more details
    unstable.nodejs-14_x # v14.16.1
    (unstable.yarn.override { nodejs = nodejs-14_x; }) # v1.22.10

    ## python is required by node module bcrypto
    python3

    # test Github automation
    unstable.act

    # testing utilities
    websocat

    # custom pkg groups
    macosPkgs
    linuxPkgs
  ];
}
