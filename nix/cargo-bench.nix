{ 
mkCargoDerivation
}:

{ cargoArtifacts

, ...
}@origArgs:
let
  args = builtins.removeAttrs origArgs [
    "cargoExtraArgs"
  ];
in
mkCargoDerivation (args // {
  inherit cargoArtifacts;
  pnameSuffix = "-bench";

  buildPhaseCargoCommand = "cargo bench --locked -F testing";

  nativeBuildInputs = (args.nativeBuildInputs or [ ]);
})