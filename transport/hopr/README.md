# hopr-transport

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

The application representing the entire physical transport functionality, including:

- p2p
- protocol
- session

## Tracing benchmark

On an Apple M4 Max, two release benchmark rounds (50 samples each) measured capacity-one queue
round trips with compact span storage at 8.28 ns with INFO logging and 38.66 ns with DEBUG logging,
down from 18.49 ns and 47.69 ns respectively. The added cost over a plain queue was 2.38 ns and
32.75 ns; active spans require one allocation. These isolated measurements do not establish
end-to-end throughput. Reproduce with `cargo bench -p hopr-transport --features testing,telemetry,runtime-tokio,serde --bench routing_tracing_bench --profile release`, setting
`HOPR_BENCH_LOG=info` or `debug` (formatted output goes to a sink).
