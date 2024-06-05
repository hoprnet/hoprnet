# A simplest nix shell file with the project dependencies and 
# a cross-compilation support.
{ localSystem ? builtins.currentSystem
, crossSystem ? null
}:
let
  pkgs = import ./nix {
    inherit localSystem crossSystem;
  };

in
pkgs.mkShell {
  # Native project dependencies like build utilities and additional routines 
  # like container building, linters, etc.
  nativeBuildInputs = with pkgs.pkgsBuildHost; [
    # Rust
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    sccache
    # Will add some dependencies like libiconv
    rustBuildHostDependencies

  ];
  # Libraries essential to build the service binaries
  buildInputs = with pkgs; [
    # Enable Rust cross-compilation support
    rustCrossHook
    #gcc_multi
  ];

  GCCMULTI_VERSION = "9";
  # You can specify a default GCC version to use

  # For cross-compilation, specify target systems
  GCCMULTI_TARGETS = [ "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];

  packages = with pkgs; [
    # testing utilities
    jq
    yq-go
    curl
    bash
    gnumake
    which

    #rust-bin.stable.latest.default
    #cmake
    #clang
    #libiconv
    #zlib
    #protobuf
    #git
    #gnupg
    #nixfmt
    #solc
    #foundry
    #gcc
    #binutils

    # github integration
    gh

    # test Github automation
    act

    # documentation utilities
    #pandoc
    #swagger-codegen3

    # docker image inspection and handling
    #dive
    #skopeo

    # test coverage generation
    #lcov

    ## python is required by integration tests
    #python39
    #python39Packages.venvShellHook

  ];
  TREE_SITTER_STATIC_BUILD = 1;
  # Prettify shell prompt
  shellHook = "${pkgs.crossBashPrompt}";
  # Use sscache to improve rebuilding performance
  env.RUSTC_WRAPPER = "sccache";

}
