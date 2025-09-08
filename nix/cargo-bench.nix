{
  mkCargoDerivation,
}:

{
  cargoArtifacts,

  ...
}@origArgs:
let
  args = builtins.removeAttrs origArgs [
    "cargoExtraArgs"
  ];
in
mkCargoDerivation (
  args
  // {
    inherit cargoArtifacts;
    pnameSuffix = "-bench";

    buildPhaseCargoCommand = "cargo bench --locked";

    nativeBuildInputs = (args.nativeBuildInputs or [ ]);

    RUST_BACKTRACE = "full";
  }
)
