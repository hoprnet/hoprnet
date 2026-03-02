# AI Agent Instructions

This file provides guidance for AI coding agents working with this repository.

All AI configuration lives in the [`.ai/`](.ai/) directory.

## Instructions

- [.ai/INSTRUCTIONS.md](.ai/INSTRUCTIONS.md) — Development guidelines, architecture, conventions
- [.ai/rust.md](.ai/rust.md) — Rust-specific rules (apply when editing `*.rs` files)

## Tool-Specific Wiring

| Tool | Entry point | Purpose |
|------|-------------|----------|
| Claude Code | [CLAUDE.md](CLAUDE.md) | Auto-loaded; references `.ai/` |
| GitHub Copilot | [.github/copilot-instructions.md](.github/copilot-instructions.md) | Auto-loaded; references `.ai/INSTRUCTIONS.md` |
| Copilot (Rust) | [.github/instructions/rust.instructions.md](.github/instructions/rust.instructions.md) | `applyTo: **/*.rs`; references `.ai/rust.md` |
