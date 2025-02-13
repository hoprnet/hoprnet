fixupCFlagsForDarwin() {
  # Because it’s getting called from a Darwin stdenv, gcc will pick up on
  # Darwin-specific flags, and it will barf and die on -iframework in
  # particular. Strip them out, so hoprd can compile.
  cflagsFilter='s|-F[^ ]*||g;s|-iframework [^ ]*||g;s|-isystem [^ ]*||g;s|  *| |g'

  # The `CoreFoundation` reference is added by `linkSystemCoreFoundationFramework` in the
  # Apple SDK’s setup hook. Remove that because gcc will fail due to file not found.
  ldFlagsFilter='s|/nix/store/[^-]*-apple-framework-CoreFoundation[^ ]*||g'

  # `cc-wrapper.sh`` supports getting flags from a system-specific salt. While it is currently a
  # tuple, that’s not considered a stable interface, so the derivation will provide them.
  export NIX_CFLAGS_COMPILE_@darwinSuffixSalt@=${NIX_CFLAGS_COMPILE-}
  export NIX_LDFLAGS_@darwinSuffixSalt@=${NIX_LDFLAGS-}

  echo removing @darwinSuffixSalt@-specific flags from NIX_CFLAGS_COMPILE @targetSuffixSalt@
  export NIX_CFLAGS_COMPILE_@targetSuffixSalt@+="$(sed "$cflagsFilter" <<< "$NIX_CFLAGS_COMPILE")"
  echo removing @darwinSuffixSalt@-specific flags from NIX_LDFLAGS for @targetSuffixSalt@
  export NIX_LDFLAGS_@targetSuffixSalt@+="$(sed "$ldFlagsFilter;$cflagsFilter" <<< "$NIX_LDFLAGS")"

  # Make sure the global flags aren’t accidentally influencing the platform-specific flags.
  export NIX_CFLAGS_COMPILE=""
  export NIX_LDFLAGS=""
}

# This is pretty hacky, but this hook _must_ run after `linkSystemCoreFoundationFramework`.
function runFixupCFlagsForDarwinLast() {
  preConfigureHooks+=(fixupCFlagsForDarwin)
}

if [ -z "${dontFixupCFlagsForDarwin-}" ]; then
  postUnpackHooks+=(runFixupCFlagsForDarwinLast)
fi
