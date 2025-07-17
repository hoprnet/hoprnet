{
  pkgs,
  hoprd,
  hopli,
}:

let
  mkManPage =
    {
      pname,
      binary,
      description,
    }:
    pkgs.stdenv.mkDerivation {
      name = "${pname}-man";

      nativeBuildInputs = [ pkgs.help2man ];
      LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH";

      buildCommand = ''
        mkdir -p $out/share/man/man1

        # Generate man page using help2man
        help2man \
          --name="${description}" \
          --no-info \
          --output=$out/share/man/man1/${pname}.1 \
          ${binary}/bin/${pname}

        # Compress the man page
        gzip $out/share/man/man1/${pname}.1
      '';
    };
in
{
  hoprd-man = mkManPage {
    pname = "hoprd";
    binary = hoprd;
    description = "HOPR node executable";
  };

  hopli-man = mkManPage {
    pname = "hopli";
    binary = hopli;
    description = "HOPR CLI helper tool";
  };
}
