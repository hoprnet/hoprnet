{
  Cmd ? [ ],
  Entrypoint ? [ ],
  env ? [ ],
  extraContents ? [ ],
  extraPorts ? { },
  fakeRootCommands ? null,
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
      gnugrep # Used for searching env variables
      gnutar # Used to extract the database files from the docker container

    ]
    ++ extraContents;
  Env = [
    "NO_COLOR=true" # suppress colored log output
    # "RUST_LOG=info"   # 'info' level is set by default with some spamming components set to override
    "RUST_BACKTRACE=full"
    "LD_LIBRARY_PATH=${libPath}"
  ]
  ++ env;
  ExposedPorts = extraPorts;
in
pkgs.dockerTools.buildLayeredImage (
  {
    inherit name contents;
    tag = "latest";
    # breaks binary reproducibility, but makes usage easier
    created = "now";
    config = {
      inherit
        Cmd
        Entrypoint
        Env
        ExposedPorts
        ;
    };
  }
  // (
    if fakeRootCommands != null then
      {
        enableFakechroot = true;
        inherit fakeRootCommands;
      }
    else
      { }
  )
)
