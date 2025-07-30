{
  Cmd ? [ ],
  Entrypoint,
  env ? [ ],
  extraContents ? [ ],
  name,
  pkgs,
}:
let
  libPath = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
  contents =
    with pkgs;
    [
      bash
      cacert
      coreutils
      findutils
      iana-etc
      nettools
    ]
    ++ extraContents;
  Env = [
    "NO_COLOR=true" # suppress colored log output
    # "RUST_LOG=info"   # 'info' level is set by default with some spamming components set to override
    "RUST_BACKTRACE=full"
    "LD_LIBRARY_PATH=${libPath}"
  ]
  ++ env;
in
pkgs.dockerTools.buildLayeredImage {
  inherit name contents;
  tag = "latest";
  # breaks binary reproducibility, but makes usage easier
  created = "now";
  config = { inherit Cmd Entrypoint Env; };
}
