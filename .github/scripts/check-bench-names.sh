#!/usr/bin/env bash
set -euo pipefail

errors=()
for cargo_toml in "$@"; do
  while IFS= read -r bench_name; do
    if [[ -n $bench_name && ! $bench_name =~ _bench$ ]]; then
      errors+=("$cargo_toml: bench name '${bench_name}' must end with '_bench'")
    fi
  done < <(awk '/^\[\[bench\]\]/{found=1} found && /^name *=/{gsub(/^name *= *"/, ""); gsub(/".*/, ""); print; found=0}' "$cargo_toml")
done

if [[ ${#errors[@]} -gt 0 ]]; then
  echo "Benchmark naming violation: all [[bench]] names must end with '_bench'"
  for err in "${errors[@]}"; do
    echo "  $err"
  done
  exit 1
fi
