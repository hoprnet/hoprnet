# hopr-transport

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

The application representing the entire physical transport functionality, including:

- p2p
- protocol
- session

## Tracing benchmark

Release benchmarks on an Apple M4 Max compared `e5430c7987` against PR base master
`900038bbe4` (three alternating rounds, 50 samples, INFO/DEBUG logs to a sink). Rebuilding 16
candidates took 5.31% longer at INFO and 8.60% longer at DEBUG; some smaller routing/session
differences changed direction across rounds. Base plain queue → current context queue times were
5.92 → 8.65 ns at INFO and 5.86 → 37.53 ns at DEBUG. This comparison includes the graph dependency
change and does not measure end-to-end throughput. Run
`cargo bench -p hopr-transport --features testing,telemetry,runtime-tokio,serde --bench routing_tracing_bench --profile release`
with `HOPR_BENCH_LOG=info` or `debug`. Separately, two alternating rounds on `0daf2e6682` showed
that retaining TRACE in release builds added 5.49% (31 ns) to cached forward routing and 6.73%
(72 ns) to cached forward routing with two return paths at INFO, compared with the build capped at DEBUG.
