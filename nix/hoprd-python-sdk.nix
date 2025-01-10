{pkgs}:
pkgs.stdenv.mkDerivation {
  pname = "hoprd-python-sdk";
  version = hoprdCrateInfo.version;
  src = fs.toSource {

  buildPhase = ''
  '';

  
  }
