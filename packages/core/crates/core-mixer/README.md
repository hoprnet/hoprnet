# Mixer

Migrate the basic packet mixer functionality into Rust as a fully native async solution with no input limits for packet ingress by creating a perpetual stream of packet futures with the first finalized (timed-out) futures being streamed first.

## Detail

A native Rust futures infrastructure [`futures::stream::FuturesUnordered`](https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html#) is used to internally cache and concurrently trigger all currently timed-out packets to produce a `std::future::Stream` implementation further usable in the JavaScript code instead of the legacy `AsyncIterator`.

The `then_concurrently` method extends every `Stream` to add the desired functionality to use the concurrent execution capability of `FuturesUnordered`.
