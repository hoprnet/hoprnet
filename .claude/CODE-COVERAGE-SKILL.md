# Cargo Llvm-Cov Skill

This skill provides instructions for running code coverage analysis using `cargo llvm-cov` in the HOPR project.

## Quick Reference

```bash
# Run coverage for a specific crate using nextest (preferred)
cargo llvm-cov nextest --package <crate> --lib --bins

# Get LCOV output for per-file analysis
cargo llvm-cov nextest --package <crate> --lib --bins --lcov --output-path /tmp/coverage.lcov
```

## Detailed Workflow

### 1. Set Working Directory

Always set the working directory to the project root first:

```bash
cargo set-working-directory -p <crate>
```

Or use the MCP tool:

```
mcp__cargo__set_working_directory(path: "/home/luc/repos/hopr/hoprnet-2")
```

### 2. Build the Crate

Ensure the crate is built before running coverage:

```bash
cargo build --package <crate>
```

### 3. Run Llvm-Cov

Basic command for a specific package:

```bash
cargo llvm-cov nextest --package hopr-transport-session --lib --bins
```

Flags:

- `--lib`: Include library code
- `--bins`: Include binaries

### 4. Generate LCOV for Per-File Analysis

To get per-file coverage breakdown, generate LCOV format:

```bash
cargo llvm-cov nextest --package <crate> --lib --bins --lcov --output-path /tmp/coverage.lcov
```

### 5. Parse Results

Use Python to extract per-file coverage from LCOV:

```python
import re

files_data = {}
current_file = None

with open('/tmp/coverage.lcov', 'r') as f:
    for line in f:
        line = line.strip()
        if line.startswith('SF:'):
            path = line[3:]
            filename = path
            current_file = filename
            if current_file not in files_data:
                files_data[current_file] = {'covered': 0, 'total': 0}
        elif line.startswith('LH:'):
            if current_file:
                files_data[current_file]['covered'] = int(line[3:])
        elif line.startswith('LF:'):
            if current_file:
                files_data[current_file]['total'] = int(line[3:])
        elif line.startswith('end_of_record'):
            current_file = None

results = []
for fname, data in files_data.items():
    if data['total'] > 0:
        pct = (data['covered'] / data['total']) * 100
        results.append((fname, data['covered'], data['total'], pct))

results.sort(key=lambda x: x[3], reverse=True)

print(f"{'File':<35} {'Covered':>10} {'Total':>10} {'Coverage':>10}")
print("-" * 70)
for fname, covered, total, pct in results:
    print(f"{fname:<35} {covered:>10} {total:>10} {pct:>9.1f}%")

total_covered = sum(x[1] for x in results)
total_total = sum(x[2] for x in results)
total_pct = (total_covered / total_total * 100) if total_total > 0 else 0
print("-" * 70)
print(f"{'TOTAL':<35} {total_covered:>10} {total_total:>10} {total_pct:>9.1f}%")
```

## Common Issues

- **"--output-dir may not be used together with --cobertura"**: Don't use both flags together. Use `--cobertura` or `--lcov` without `--output-dir`.
- **Empty results**: Ensure the crate is built first before running llvm-cov.
- **Missing permissions**: Ensure `cargo:*` is allowed in settings.json.

## Example Session

```
# Set working directory
cargo set-working-directory -p hopr-transport-session

# Build
cargo build --package hopr-transport-session

# Run coverage with LCOV
cargo llvm-cov --package hopr-transport-session --lib --bins --tests --lcov --output-path /tmp/coverage.lcov

# Parse results with Python script above
```
