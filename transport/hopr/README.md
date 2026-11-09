# hopr-transport

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

The application representing the entire physical transport functionality, including:

- p2p
- protocol
- session

## Tracing benchmark

Release benchmarks on an Apple M4 Max compared `5ec1d5fef3` with PR base master
`3e0395cc33` (three rounds, 50 samples, paired per operation, INFO/DEBUG logs to a sink).
Cached forward routing with two return paths took 12.15% longer at INFO and 32.58% less time
at DEBUG; DEBUG rebuilds took 13.49% less time. Base plain queue → branch context queue times
were 5.77 → 8.29 ns at INFO and 5.79 → 35.07 ns at DEBUG. The comparison includes the graph dependency
and release TRACE configuration changes; these component fixtures exclude network latency and
consumer-span restoration. Run
`cargo bench -p hopr-transport --features testing,telemetry,runtime-tokio,serde --bench routing_tracing_bench --profile release`
with `HOPR_BENCH_LOG=info` or `debug`.
