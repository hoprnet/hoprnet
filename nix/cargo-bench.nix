{ 
mkCargoDerivation
}:

{ cargoArtifacts
, cargoExtraArgs ? "--locked -F testing"
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

  buildPhaseCargoCommand = "cargoWithProfile bench ${cargoExtraArgs}";

  nativeBuildInputs = (args.nativeBuildInputs or [ ]);
})