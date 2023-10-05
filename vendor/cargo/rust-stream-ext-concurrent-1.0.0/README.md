# rust-stream-ext-concurrent

Concurrent behavior extensions for any `futures::stream::Stream` objects.

## Detail

A native Rust futures infrastructure [`futures::stream::FuturesUnordered`](https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html#) is used to internally cache and enable concurrent processing of `Stream` objects.

### Supported extensions
- `then_concurrent` method extends every `Stream` to add the desired functionality to use the concurrent execution capability of `FuturesUnordered`
