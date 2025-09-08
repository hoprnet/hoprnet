# man-pages.nix - Manual page generation
#
# Generates manual pages (man pages) for HOPR binaries using help2man.
# Creates documentation from the --help output of the compiled binaries.

{
  pkgs,
  hoprd, # HOPRD daemon binary package
  hopli, # HOPLI CLI tool binary package
}:

let
  # Create a manual page derivation from a binary
  # Extracts help information and formats it as a standard man page
  mkManPage =
    {
      pname, # Package name for the manual page
      binary, # Binary executable to generate documentation from
      description, # Brief description of the tool
    }:
    pkgs.stdenv.mkDerivation {
      name = "${pname}-man";

      # Tools needed for generating manual pages
      nativeBuildInputs = [ pkgs.help2man ];
      # Ensure OpenSSL libraries are available for binary execution
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
