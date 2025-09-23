# Channel Metrics Benchmarks

This directory contains benchmarks for measuring the performance impact of channel monitoring in `hopr-async-runtime`.

## Quick Start

```bash
# Run all configurations using the helper script
./benches/bench_all_configs.sh

# Run single configuration quickly  
./benches/bench_all_configs.sh -c tokio_prometheus -q

# Run specific benchmark directly
cargo bench -p hopr-async-runtime channel_metrics_simple
```

## Files

- **`channel_metrics_simple.rs`** - Main benchmark suite (works with all configurations)
  - Contains comprehensive documentation and usage examples
  - Includes result interpretation guidelines
  - Supports all feature combinations reliably

- **`bench_all_configs.sh`** - Helper script for running all configurations
  - Automated testing across all feature combinations
  - Color-coded output and result management
  - Includes help with `./bench_all_configs.sh --help`

## Documentation

All detailed documentation, usage examples, and result interpretation guidelines are embedded directly in the benchmark files as Rust doc comments. View with:

```bash
# View benchmark documentation
cargo doc -p hopr-async-runtime --open --document-private-items
```

Or read the source files directly - they contain comprehensive inline documentation.

## Supported Configurations

- ✅ `runtime-tokio` + `prometheus`
- ✅ `runtime-tokio` (no prometheus)  
- ✅ `runtime-futures` + `prometheus`
- ✅ `runtime-futures` (no prometheus)

All benchmarks are designed to work reliably across these configurations.