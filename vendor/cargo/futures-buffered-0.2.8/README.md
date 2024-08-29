# futures-buffered

This project provides several future structures, all based around the `FuturesUnorderedBounded` primtive.

Much like [`futures::FuturesUnordered`](https://docs.rs/futures/0.3.25/futures/stream/struct.FuturesUnordered.html), this is a thread-safe, `Pin` friendly, lifetime friendly, concurrent processing stream.

This primtive is different to `FuturesUnordered` in that `FuturesUnorderedBounded` has a fixed capacity for processing count. This means it's less flexible, but produces better memory efficiency.

However, we also provide a `FuturesUnordered` which allocates larger `FuturesUnorderedBounded`
automatically to mitigate these inflexibilities. This is based on a triangular-array concept
to amortise the cost of allocating (much like with a Vec) without violating `Pin` constraints.

## Benchmarks

### Speed

Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:

```
FuturesUnorderedBounded    [339.9 ms  364.7 ms  380.6 ms]
futures::FuturesUnordered  [377.4 ms  391.4 ms  406.3 ms]
                           [min         mean         max]
```

### Memory usage

Running 512000 `Ready<i32>` futures with 256 concurrent jobs.

- count: the number of times alloc/dealloc was called
- alloc: the number of cumulative bytes allocated
- dealloc: the number of cumulative bytes deallocated

```
futures::FuturesUnordered
    count:    1,024,004
    alloc:    40.96 MB
    dealloc:  40.96 MB

FuturesUnorderedBounded
    count:    4
    alloc:    8.28 KB
    dealloc:  8.28 KB
```

### Conclusion

As you can see, `FuturesUnorderedBounded` massively reduces you memory overhead while providing a small performance gain. Perfect for if you want a fixed batch size

## Examples

```rust
// create a tcp connection
let stream = TcpStream::connect("example.com:80").await?;

// perform the http handshakes
let (mut rs, conn) = conn::handshake(stream).await?;
runtime.spawn(conn);

/// make http request to example.com and read the response
fn make_req(rs: &mut SendRequest<Body>) -> ResponseFuture {
    let req = Request::builder()
        .header("Host", "example.com")
        .method("GET")
        .body(Body::from(""))
        .unwrap();
    rs.send_request(req)
}

// create a queue that can hold 128 concurrent requests
let mut queue = FuturesUnorderedBounded::new(128);

// start up 128 requests
for _ in 0..128 {
    queue.push(make_req(&mut rs));
}
// wait for a request to finish and start another to fill its place - up to 1024 total requests
for _ in 128..1024 {
    queue.next().await;
    queue.push(make_req(&mut rs));
}
// wait for the tail end to finish
for _ in 0..128 {
    queue.next().await;
}
```

```rust
use futures_buffered::join_all;

async fn foo(i: u32) -> u32 { i }

let futures = vec![foo(1), foo(2), foo(3)];

assert_eq!(join_all(futures).await, [1, 2, 3]);
```
