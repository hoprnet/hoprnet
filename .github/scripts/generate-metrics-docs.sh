#!/usr/bin/env bash
# Lint: ensures every Prometheus metric defined in Rust code is documented in
# METRICS.md, and that METRICS.md does not reference non-existing metrics.
#
# Usage:  ./.github/scripts/generate-metrics-docs.sh              (lint — verify sync)

#         ./.github/scripts/generate-metrics-docs.sh --generate    (print markdown table to stdout)
#         just generate-metrics-docs                               (via justfile recipe)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
METRICS_DOC="$REPO_ROOT/METRICS.md"

# ── Helper: extract metrics from code ────────────────────────────────────────
# Outputs tab-separated rows:  name \t type \t description \t detail
# Requires 8 lines of trailing context to capture buckets/keys on later lines.
extract_metrics() {
  git -C "$REPO_ROOT" grep -A8 -E '(Simple|Multi)(Counter|Gauge|Histogram)::new\(' -- '*.rs' 2>/dev/null |
    gawk '
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
  ' |
    sort -t$'\t' -k1
}

# ── --generate: print a markdown table and exit ──────────────────────────────
if [[ ${1:-} == "--generate" ]]; then
  echo "| Name | Type | Description | Detail |"
  echo "|------|------|-------------|--------|"
  extract_metrics | while IFS=$'\t' read -r name type desc detail; do
    printf "| \`%s\` | %s | %s | %s |\n" "$name" "$type" "$desc" "$detail"
  done
  exit 0
fi

# ── Lint mode ────────────────────────────────────────────────────────────────

if [[ ! -f $METRICS_DOC ]]; then
  echo "ERROR: METRICS.md not found at $METRICS_DOC" >&2
  exit 1
fi

# Normalize a markdown table: collapse runs of whitespace around pipes so that
# column-aligned (prettified) tables compare equal to compact ones.
normalize_table() {
  sed -E 's/[ ]+\|/|/g; s/\|[ ]+/|/g; s/\|-+/|---/g'
}

# Generate expected content and compare with the actual file (whitespace-tolerant)
expected=$("$0" --generate)

if ! diff -q <(echo "$expected" | normalize_table) <(normalize_table <"$METRICS_DOC") >/dev/null 2>&1; then
  echo "ERROR: METRICS.md is out of date. Differences:"
  diff -u <(normalize_table <"$METRICS_DOC") <(echo "$expected" | normalize_table) | head -40
  echo ""
  echo "Run:  $0 --generate > METRICS.md"
  exit 1
fi

count=$(echo "$expected" | tail -n +3 | wc -l | tr -d ' ')
echo "OK: All $count metrics are in sync between code and METRICS.md."
