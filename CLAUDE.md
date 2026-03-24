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
6. Run `cargo test --no-run` and `cargo clippy` to verify that everything builds correctly.
7. Run tests (unit and integration separately, see below).
8. Once everything is finished and passing, commit the changes with a descriptive title using the Conventional Commits notation.

## Tests

### Unit Tests

```bash
cargo test --lib                        # All unit tests
cargo test --lib -p <crate>             # Single crate unit tests
```

### Integration Tests

Integration tests **must** run single-threaded due to shared cluster resources:

```bash
cargo test --test '*' -- --test-threads=1           # All integration tests
cargo test -p <crate> --test <test_name> -- --test-threads=1  # Single test file
```

For `hopr-builder` integration tests specifically:

```bash
cargo test -p hopr-builder --test transport_tickets -- --test-threads=1
cargo test -p hopr-builder --test transport_session -- --test-threads=1
cargo test -p hopr-builder --test chain_operations-size2 -- --test-threads=1
cargo test -p hopr-builder --test chain_operations-size3 -- --test-threads=1
```

## Permissions

See `.claude/settings.json` for allowed commands.
