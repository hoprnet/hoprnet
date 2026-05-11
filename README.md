<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img align="middle" width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>

  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
</p>

<p align="center">
  <a href="https://codecov.io/gh/hoprnet/hoprnet">
    <img src="https://codecov.io/gh/hoprnet/hoprnet/branch/master/graph/badge.svg" alt="codecov">
  </a>
  <a href="https://bencher.dev/console/projects/hoprnet/perf">
    <img src="https://img.shields.io/badge/Performance-Bencher-6366f1?style=flat&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iI2ZmZiIgZD0iTTEyIDJMMiA3djEwbDEwIDUgMTAtNVY3bC0xMC01ek00IDguOTVsOC00IDggNGwtOCA0LTgtNHptOCAxMC4xNWwtNi0zVjEwbDYgM3Y2LjE1em04LTMuMTVsbC02IDNWMTBsNi0zdjYuMTV6Ii8+PC9zdmc+" alt="Bencher Performance Dashboard">
  </a>
</p>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [About](#about)
- [Develop](#develop)
  - [Nix environment setup](#nix-environment-setup)
    - [Nix flake outputs](#nix-flake-outputs)
    - [Code Formatting](#code-formatting)
    - [Code Linting](#code-linting)
- [Test](#test)
  - [Github Actions CI](#github-actions-ci)
- [Code Coverage](#code-coverage)
- [Profiling \& Instrumentation](#profiling--instrumentation)
  - [Profiling Criterion benchmarks via `flamegraph`](#profiling-criterion-benchmarks-via-flamegraph)
    - [Prerequisites](#prerequisites)
    - [Profiling the benchmarking binaries](#profiling-the-benchmarking-binaries)
- [Contact](#contact)
- [License](#license)

## About

This repository contains the core HOPR protocol library and supporting crates.

1. [hopr-lib](https://hoprnet.github.io/hoprnet/hopr_lib/index.html)
   - A fully self-contained referential implementation of the HOPR protocol over a libp2p based connection mechanism that can be incorporated into other projects as a transport layer.

The `hoprd` daemon (HOPR node with REST API) lives in the [hoprd repository](https://github.com/hoprnet/hoprd).

## Develop

Either setup `nix` and `flake` to use the nix environment, or [install Rust toolchain](https://www.rust-lang.org/tools/install) from the `rust-toolchain.toml`, as well as `foundry-rs` binaries (`forge`, `anvil`).

### Nix environment setup

Install `nix` from the official website at [https://nix.dev/install-nix.html](https://nix.dev/install-nix.html).

Create a nix configuration file at `~/.config/nix/nix.conf` with the following content:

```nix
experimental-features = nix-command flakes
```

Install the `nix-direnv` package to introduce the `direnv`:

```bash
nix-env -i nix-direnv
```

Append the following line to the shell rc file (depending on the shell used it can be `~\.zshrc`, `~\.bashrc`, `~\.cshrc`, etc.). Modify the `<shell>` variable inside the below command with the currently used (`zsh`, `bash`, `csh`, etc.):

```bash
eval "$(direnv hook <shell>)"
```

From within the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository's directory, execute the following command.

```bash
direnv allow .
```

#### Nix flake outputs

We provide a couple of packages, apps and shells to make building and
development easier. You may get the full list like so:

```bash
nix flake show
```

#### Code Formatting

All nix, rust, solidity and python code can be automatically formatted:

```bash
nix fmt
```

These formatters are also automatically run as a Git pre-commit check.

#### Code Linting

All linters can be executed via a Nix flake helper app:

```bash
nix run .#check
```

This will in particular run `clippy` for the entire Rust codebase.

## Test

Run all tests: `cargo test`.

Run only unit tests: `cargo test --lib`

### Github Actions CI

We run a fair amount of automation using Github Actions. Too see the full list of workflows checkout [workflow docs](./.github/workflows/README.md)

## Code Coverage

Coverage reports are generated using LLVM source-based instrumentation and uploaded to [Codecov](https://codecov.io/gh/hoprnet/hoprnet). See [docs/coverage.md](docs/coverage.md) for workspace-wide and single-crate usage.

## Profiling & Instrumentation

### Profiling Criterion benchmarks via `flamegraph`

#### Prerequisites

- `perf` installed on the host system
- flamegraph (install via e.g. `cargo install flamegraph`)

#### Profiling the benchmarking binaries

1. Perform a build of your chosen benchmark with `--no-rosegment` linker flag:

   ```
   RUSTFLAGS="-Clink-arg=-fuse-ld=lld -Clink-arg=-Wl,--no-rosegment" cargo bench --no-run -p hopr-crypto-packet
   ```

   Use `mold` instead of `lld` if needed.

2. Find the built benchmarking binary and check if it contains debug symbols:

   ```
   readelf -S target/release/deps/packet_benches-ce70d68371e6d19a | grep debug
   ```

   The output of the above command should contain AT LEAST: `.debug_line`, `.debug_info` and `.debug_loc`

3. Run `flamegraph` on the benchmarking binary of a selected benchmark with a fixed profile time (e.g.: 30 seconds):
   ```
   flamegraph -- ./target/release/deps/packet_benches-ce70d68371e6d19a --bench --exact packet_sending_no_precomputation/0_hop_0_surbs --profile-time 30
   ```
4. The `flamegraph.svg` will be generated in the project root directory and can be opened in a browser.

## Contact

- [X](https://x.com/hoprnet)
- [Telegram](https://t.me/hoprnet)
- [Medium](https://medium.com/hoprnet)
- [Reddit](https://www.reddit.com/r/HOPR/)
- [Email](mailto:contact@hoprnet.org)
- [Discord](https://discord.gg/5FWSfq7)
- [Youtube](https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA)

## License

[GPL v3](https://github.com/hoprnet/hoprnet/blob/master/LICENSE) © HOPR Association

[1]: https://nixos.org/learn.html
[2]: https://github.com/nektos/act
