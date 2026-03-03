# AI Agent Instructions

This file provides guidance for AI coding agents working with this repository.

Canonical AI configuration lives in the [`.claude/`](.claude/) directory.

## Instructions

- [.claude/INSTRUCTIONS.md](.claude/INSTRUCTIONS.md) — Development guidelines, architecture, conventions
- [.claude/rust.md](.claude/rust.md) — Rust-specific rules (apply when editing `*.rs` files)

## Tool-Specific Wiring

| Tool           | Entry point                                                                            | Purpose                                           |
| -------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------- |
| Claude Code    | [CLAUDE.md](CLAUDE.md)                                                                 | Auto-loaded; references `.claude/`                |
| GitHub Copilot | [.github/.copilot-instructions.md](.github/.copilot-instructions.md)                   | Auto-loaded; references `.claude/INSTRUCTIONS.md` |
| Copilot (Rust) | [.github/instructions/rust.instructions.md](.github/instructions/rust.instructions.md) | `applyTo: **/*.rs`; references `.claude/rust.md`  |
