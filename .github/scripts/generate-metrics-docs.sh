#!/usr/bin/env bash
# Lint: ensures every Prometheus metric defined in Rust code is documented in
# METRICS.md, and that METRICS.md does not reference non-existing metrics.
#
# Usage:  ./.github/scripts/generate-metrics-docs.sh              (lint — verify sync)
#         ./.github/scripts/generate-metrics-docs.sh --changed     (only check changed files)
#         ./.github/scripts/generate-metrics-docs.sh --generate    (print markdown table to stdout)
#         just generate-metrics-docs                               (via justfile recipe)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
METRICS_DOC="$REPO_ROOT/METRICS.md"

# ── Helper: extract metrics from code ────────────────────────────────────────
# Outputs tab-separated rows:  name \t type \t description \t detail
# Requires 8 lines of trailing context to capture buckets/keys on later lines.
extract_metrics() {
  git -C "$REPO_ROOT" grep -A8 -E '(Simple|Multi)(Counter|Gauge|Histogram)::new\(' -- '*.rs' 2>/dev/null \
  | gawk '
    /^misc\/metrics\// { next }

    # ── new ::new( call → flush previous metric and start fresh ──
    match($0, /(Simple|Multi)(Counter|Gauge|Histogram)::new\(/, t) {
      if (name != "") {
        detail = ""
        if (keys != "") detail = detail "keys: " keys
        if (buckets != "") { if (detail != "") detail = detail "; "; detail = detail "buckets: " buckets }
        printf "%s\t%s\t%s\t%s\n", name, type, desc, detail
      }
      name = ""; type = ""; desc = ""; buckets = ""; keys = ""
      type = t[1] t[2]
      # Try to extract name + description + keys + buckets from the same line
      if (match($0, /"(hopr_[a-z0-9_]+)"/, m)) name = m[1]
      if (name != "" && match($0, /hopr_[a-z0-9_]+",[ ]*"([^"]+)"/, d)) desc = d[1]
      if (name != "" && match($0, /&\[([^\]]+)\]/, k)) {
        keys = k[1]; gsub(/"/, "", keys); gsub(/,  */, ", ", keys)
      }
      if (name != "" && match($0, /vec!\[([0-9.,_ ]+)\]/, b)) {
        buckets = b[1]; gsub(/[_ ]/, "", buckets)
      }
      next
    }

    # ── group separator ──
    /^--$/ {
      if (name != "") {
        detail = ""
        if (keys != "") detail = detail "keys: " keys
        if (buckets != "") { if (detail != "") detail = detail "; "; detail = detail "buckets: " buckets }
        printf "%s\t%s\t%s\t%s\n", name, type, desc, detail
      }
      name = ""; type = ""; desc = ""; buckets = ""; keys = ""
      next
    }

    # ── first "hopr_..." string is the metric name ──
    name == "" && match($0, /"(hopr_[a-z0-9_]+)"/, m) { name = m[1]; next }

    # ── second quoted string is the description ──
    name != "" && desc == "" && match($0, /"([^"]+)"/, d) { desc = d[1]; next }

    # ── collect bucket values (vec![...]) ──
    name != "" && match($0, /vec!\[([0-9.,_ ]+)\]/, b) {
      buckets = b[1]; gsub(/[_ ]/, "", buckets); next
    }

    # ── collect named constant buckets ──
    name != "" && buckets == "" && /TIMING_BUCKETS|BUCKETS/ {
      buckets = "0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.15, 0.25, 0.5, 1.0"; next
    }

    # ── collect label keys (&["key1", "key2"]) ──
    name != "" && match($0, /&\[([^\]]+)\]/, k) {
      keys = k[1]; gsub(/"/, "", keys); gsub(/,  */, ", ", keys); next
    }

    END {
      if (name != "") {
        detail = ""
        if (keys != "") detail = detail "keys: " keys
        if (buckets != "") { if (detail != "") detail = detail "; "; detail = detail "buckets: " buckets }
        printf "%s\t%s\t%s\t%s\n", name, type, desc, detail
      }
    }
  ' \
  | sort -t$'\t' -k1
}

# ── --generate: print a markdown table and exit ──────────────────────────────
if [[ "${1:-}" == "--generate" ]]; then
  echo "| Name | Type | Description | Detail |"
  echo "|------|------|-------------|--------|"
  extract_metrics | while IFS=$'\t' read -r name type desc detail; do
    printf "| \`%s\` | %s | %s | %s |\n" "$name" "$type" "$desc" "$detail"
  done
  exit 0
fi

# ── Lint mode ────────────────────────────────────────────────────────────────

if [[ ! -f "$METRICS_DOC" ]]; then
  echo "ERROR: METRICS.md not found at $METRICS_DOC" >&2
  exit 1
fi

# 1. Metric names from code
code_metrics=$(extract_metrics | cut -f1 | sort -u)

# 2. Metric names from the METRICS.md table (backtick-delimited in first column)
doc_metrics=$(
  grep -oP '`\Khopr_[a-z0-9_]+(?=`)' "$METRICS_DOC" \
  | sort -u
)

# 3. Compare
exit_code=0

missing_from_doc=$(comm -23 <(echo "$code_metrics") <(echo "$doc_metrics") || true)
if [[ -n "$missing_from_doc" ]]; then
  echo "ERROR: Metrics defined in code but missing from METRICS.md:"
  pattern=$(echo "$missing_from_doc" | paste -sd'|' -)
  git -C "$REPO_ROOT" grep -n -E "\"($pattern)\"" -- '*.rs' 2>/dev/null \
    | grep -v '^misc/metrics/' \
    | while IFS=: read -r file line _rest; do
        echo "  - $(echo "$_rest" | grep -oP '"hopr_[a-z0-9_]+"' | head -1 | tr -d '"')  ($file:$line)"
      done
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
