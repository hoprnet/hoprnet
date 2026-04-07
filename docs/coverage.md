# Code Coverage

Coverage uses LLVM source-based instrumentation. CI uploads workspace-wide reports to [Codecov](https://codecov.io/gh/hoprnet/hoprnet) on every PR.

## Workspace-wide (via Nix)

This is what CI runs. It produces an LCOV report for the entire workspace:

```bash
nix build -L .#coverage-unit            # LCOV report written to ./result
```

## Single crate (local, from the dev shell)

To generate a coverage report for a single crate, set the `CRATE` variable and run:

```bash
nix develop --command bash -c '
  OUTDIR=/tmp/hopr-coverage
  CRATE=hopr-transport-mixer            # ← change to target crate
  CRATE_US=${CRATE//-/_}

  rm -rf "$OUTDIR" && mkdir -p "$OUTDIR"

  # 1. Build and run tests with coverage instrumentation
  LLVM_PROFILE_FILE="$OUTDIR/%p_%m.profraw" \
  RUSTFLAGS="-C instrument-coverage" \
    cargo test -p "$CRATE" --lib

  # 2. Merge raw profiles
  llvm-profdata merge -sparse "$OUTDIR"/*.profraw -o "$OUTDIR/coverage.profdata"

  # 3. Find the test binary
  BINARY=$(find target/debug/deps -name "${CRATE_US}-*" -type f \
    ! -name "*.d" ! -name "*.rmeta" ! -name "*.o" \
    | while read f; do file "$f" | grep -q "Mach-O\|ELF" && echo "$f"; done | head -1)

  # 4. Print summary to terminal
  llvm-cov report "$BINARY" \
    --instr-profile="$OUTDIR/coverage.profdata" \
    --ignore-filename-regex="\.cargo|rustc"

  # 5. Export LCOV and generate HTML report
  llvm-cov export "$BINARY" \
    --instr-profile="$OUTDIR/coverage.profdata" \
    --ignore-filename-regex="\.cargo|rustc" \
    --format=lcov > "$OUTDIR/coverage.lcov"

  genhtml "$OUTDIR/coverage.lcov" \
    --output-directory "$OUTDIR/html" \
    --title "$CRATE coverage"

  echo "Report: $OUTDIR/html/index.html"
'
```

Open the HTML report:

```bash
open /tmp/hopr-coverage/html/index.html        # macOS
xdg-open /tmp/hopr-coverage/html/index.html    # Linux
```

## How it works

1. `RUSTFLAGS="-C instrument-coverage"` tells `rustc` to insert LLVM instrumentation counters into the compiled code.
2. Running the test binary produces `.profraw` files containing raw counter data.
3. `llvm-profdata merge` combines multiple profraw files into a single indexed profile.
4. `llvm-cov report` prints a per-file summary (regions, functions, lines).
5. `llvm-cov export --format=lcov` converts to LCOV format for tooling (Codecov, genhtml).
6. `genhtml` (from lcov, available in the dev shell) renders a browsable HTML report.

## CI integration

The `code-coverage` job in `.github/workflows/tests.yaml` runs on every non-draft PR:

1. Builds `nix build -L .#coverage-unit` (workspace-wide LCOV report).
2. Uploads the report to Codecov using `codecov/codecov-action`.
3. Codecov posts a status check targeting the base branch coverage with 0% regression tolerance (configured in `codecov.yml`).

## Tools available in the dev shell

| Tool | Source | Purpose |
|------|--------|---------|
| `llvm-profdata` | llvm-binutils | Merge raw profile data |
| `llvm-cov` | llvm-binutils | Generate reports from profile data |
| `genhtml` | lcov | Render LCOV as browsable HTML |
