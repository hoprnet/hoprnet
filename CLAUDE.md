# HOPR — Claude Configuration

Read `.claude/INSTRUCTIONS.md` for full development guidelines, coding standards, and project conventions.

When working on Rust files, also read `.claude/rust.md` for language-specific rules.

Claude configuration and instructions live under `.claude/`.

## Workflow

1. Before modifying code, understand the surrounding context and existing patterns.
2. For multi-step features, plan before implementing.
3. After changes, run `cargo check` (or the closest package check) to verify.
4. For Rust changes, run `cargo shear --fix -p <crate>` (never `--fix` at workspace root without `-p`) followed by `cargo check` when a cycle is finished.
5. Run `nix fmt` check.
6. Run `cargo nextest run --no-run` and `cargo clippy` to verify that everything builds correctly.
7. Run tests (unit and integration separately, see below).
8. Once everything is finished and passing, commit the changes with a descriptive title using the Conventional Commits notation.

## Tests

### Unit Tests

```bash
cargo nextest run --lib                        # All unit tests
cargo nextest run --lib -p <crate>             # Single crate unit tests
```

### Integration Tests

Integration tests **must** run single-threaded due to shared cluster resources:

```bash
cargo nextest run --test '*' -j 1                          # All integration tests
cargo nextest run -p <crate> --test <test_name> -j 1       # Single test file
```

For `hopr-lib` cluster integration tests specifically:

```bash
cargo nextest run -p hopr-lib --test transport_tickets -j 1
cargo nextest run -p hopr-lib --test transport_session -j 1
cargo nextest run -p hopr-lib --test chain_operations-size2 -j 1
cargo nextest run -p hopr-lib --test chain_operations-size3 -j 1
```

For `hopr-lib` multi-node throughput cluster tests:

```bash
cargo nextest run -p hopr-lib --features testing --test 'cluster_throughput-size3' -j 1
cargo nextest run -p hopr-lib --features testing --test 'cluster_throughput-size5' -j 1 --run-ignored all
# OFAT matrices (all variants, ~60–120 s bootstrap):
cargo nextest run -p hopr-lib --features session-client --test 'cluster_throughput-matrix' -j 1 --run-ignored all
```

For profiling the packet pipeline against a real-QUIC cluster (mock chain):

```bash
cargo run --profile profiling --features testing,profiling \
  --example cluster_throughput -- --hops 1 --mb 20 --out /tmp/flame.svg
```

For `hopr-lib` non-cluster integration tests:

```bash
cargo nextest run -p hopr-lib --test session_integration_tests -j 1
cargo nextest run -p hopr-lib --test telemetry_integration -j 1
```

## Permissions

See `.claude/settings.json` for allowed commands.
