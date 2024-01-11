# Changelog

## atomic-write-file 0.1.2

### Linux changes

* Detect whether anonymous temporary files are supported or not, and
  automatically fall back to named temporary files in case they're not.

## atomic-write-file 0.1.1

### Unix changes

* Update dependency on `nix` to version 0.27 (contributed by
  [messense](https://github.com/andreacorbellini/rust-atomic-write-file/pull/2)).

## atomic-write-file 0.1.0

* Initial release.
