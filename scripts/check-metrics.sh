#!/usr/bin/env bash
# Lint: ensures every Prometheus metric defined in Rust code is documented in
# METRICS.md, and that METRICS.md does not reference non-existing metrics.
#
# Usage:  ./scripts/check-metrics.sh              (full scan from repo root)
#         ./scripts/check-metrics.sh --changed     (only check changed files)
#         just check-metrics                       (via justfile recipe)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METRICS_DOC="$REPO_ROOT/METRICS.md"

if [[ ! -f "$METRICS_DOC" ]]; then
  echo "ERROR: METRICS.md not found at $METRICS_DOC" >&2
  exit 1
fi

# ── 1. Collect metric names from Rust source ─────────────────────────────────
# Single-pass: grep with 3 lines of trailing context (-A3), then awk extracts
# the first "hopr_..." string from each match group.  No per-match subshells.
grep_cmd=(git -C "$REPO_ROOT" grep -n -A3 -E '(Simple|Multi)(Counter|Gauge|Histogram)::new\(' -- '*.rs')

# Optionally restrict to files changed vs the merge-base of the default branch.
if [[ "${1:-}" == "--changed" ]]; then
  base=$(git -C "$REPO_ROOT" merge-base HEAD origin/main 2>/dev/null \
      || git -C "$REPO_ROOT" merge-base HEAD main 2>/dev/null \
      || echo HEAD~1)
  mapfile -t changed < <(git -C "$REPO_ROOT" diff --name-only --diff-filter=d "$base" -- '*.rs' 'METRICS.md')
  rs_files=()
  metrics_md_changed=false
  for f in "${changed[@]}"; do
    [[ "$f" == METRICS.md ]] && metrics_md_changed=true
    [[ "$f" == *.rs ]] && rs_files+=("$f")
  done
  if [[ ${#rs_files[@]} -eq 0 && "$metrics_md_changed" == false ]]; then
    echo "OK: No .rs or METRICS.md files changed — nothing to check."
    exit 0
  fi
  if [[ ${#rs_files[@]} -gt 0 ]]; then
    grep_cmd=(git -C "$REPO_ROOT" grep -n -A3 -E '(Simple|Multi)(Counter|Gauge|Histogram)::new\(' -- "${rs_files[@]}")
  else
    grep_cmd=(true)  # no .rs changes; only METRICS.md changed — still need full code scan
    grep_cmd=(git -C "$REPO_ROOT" grep -n -A3 -E '(Simple|Multi)(Counter|Gauge|Histogram)::new\(' -- '*.rs')
  fi
fi

code_metrics=$(
  "${grep_cmd[@]}" 2>/dev/null \
  | awk '
    # Skip misc/metrics test helpers
    /^misc\/metrics\// { next }
    # Look for "hopr_..." strings in the context window
    match($0, /"(hopr_[a-z0-9_]+)"/, m) { print m[1] }
  ' \
  | sort -u
)

# ── 2. Collect metric names documented in METRICS.md ─────────────────────────
doc_metrics=$(
  grep -oP '`\Khopr_[a-z0-9_]+(?=`)' "$METRICS_DOC" \
  | sort -u
)

# ── 3. Compare ───────────────────────────────────────────────────────────────
exit_code=0

missing_from_doc=$(comm -23 <(echo "$code_metrics") <(echo "$doc_metrics") || true)
if [[ -n "$missing_from_doc" ]]; then
  echo "ERROR: Metrics defined in code but missing from METRICS.md:"
  # Single git-grep to locate all missing metrics at once
  pattern=$(echo "$missing_from_doc" | paste -sd'|' -)
  git -C "$REPO_ROOT" grep -n -E "\"($pattern)\"" -- '*.rs' 2>/dev/null \
    | grep -v '^misc/metrics/' \
    | awk -F: '!seen[$0]++ { printf "  - %s  (%s:%s)\n", $3, $1, $2 }' \
    | sed "s/  - \"//; s/\".*//" \
    | while IFS= read -r loc; do echo "  $loc"; done
  # Fallback: just list names if git-grep didn't print anything useful
  if [[ $? -ne 0 ]]; then
    while IFS= read -r m; do echo "  - $m"; done <<< "$missing_from_doc"
  fi
  exit_code=1
fi

stale_in_doc=$(comm -13 <(echo "$code_metrics") <(echo "$doc_metrics") || true)
if [[ -n "$stale_in_doc" ]]; then
  echo "ERROR: Metrics documented in METRICS.md but not found in code:"
  while IFS= read -r m; do echo "  - $m"; done <<< "$stale_in_doc"
  exit_code=1
fi

if [[ $exit_code -eq 0 ]]; then
  echo "OK: All $(echo "$code_metrics" | wc -l | tr -d ' ') metrics are in sync between code and METRICS.md."
fi

exit $exit_code
