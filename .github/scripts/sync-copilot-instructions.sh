#!/usr/bin/env bash
# Sync canonical .claude/ instruction files to their Copilot counterparts.
# Called as a pre-commit hook so changes are always kept in sync.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"

changed=0

# --- .claude/INSTRUCTIONS.md → .github/.copilot-instructions.md ---
src="$REPO_ROOT/.claude/INSTRUCTIONS.md"
dst="$REPO_ROOT/.github/.copilot-instructions.md"

if [ -f "$src" ]; then
  header='<!-- Canonical source: .claude/INSTRUCTIONS.md — edit THERE, then sync this copy. -->'
  expected="$(
    printf '%s\n\n' "$header"
    cat "$src"
  )"

  if [ ! -f "$dst" ] || [ "$(cat "$dst")" != "$expected" ]; then
    printf '%s\n\n' "$header" >"$dst"
    cat "$src" >>"$dst"
    git add "$dst"
    changed=1
  fi
fi

# --- .claude/rust.md → .github/instructions/rust.instructions.md ---
src="$REPO_ROOT/.claude/rust.md"
dst="$REPO_ROOT/.github/instructions/rust.instructions.md"

if [ -f "$src" ]; then
  frontmatter='---
applyTo: "**/*.rs"
description: "Rust-specific guidelines"
---'
  header='<!-- Canonical source: .claude/rust.md — edit THERE, then sync this copy. -->'
  expected="$(
    printf '%s\n\n%s\n\n' "$frontmatter" "$header"
    cat "$src"
  )"

  if [ ! -f "$dst" ] || [ "$(cat "$dst")" != "$expected" ]; then
    printf '%s\n\n%s\n\n' "$frontmatter" "$header" >"$dst"
    cat "$src" >>"$dst"
    git add "$dst"
    changed=1
  fi
fi

if [ "$changed" -eq 1 ]; then
  echo "Synced .claude/ instructions → Copilot files"
fi
