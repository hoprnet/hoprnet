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

  buildPhaseCargoCommand = "cargoWithProfile bench -F testing -F benchmarks";

  nativeBuildInputs = (args.nativeBuildInputs or [ ]);
})