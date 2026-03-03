# HOPR — Claude Configuration

Read `.ai/INSTRUCTIONS.md` for full development guidelines, coding standards, and project conventions.

When working on Rust files, also read `.ai/rust.md` for language-specific rules.

Claude configuration and prompts live under `.ai/`.

## Workflow

1. Before modifying code, understand the surrounding context and existing patterns.
2. For multi-step features, plan before implementing.
3. After changes, run `cargo check` (or the closest package check) to verify.
4. For Rust changes, run `cargo shear --fix` followed by `cargo check` when a cycle is finished.
5. Run `nix fmt` check.
6. Run `cargo test --no-run` and `cargo clippy` to verify that everything builds correctly.
7. Run tests as specified in the tests.yaml github workflow (unit tests and integration tests separately)
8. Once everything is finished and passing, commit the changes with a descriptive title using the Conventional Commits notation

## Permissions

See `.claude/settings.json` for allowed commands.
