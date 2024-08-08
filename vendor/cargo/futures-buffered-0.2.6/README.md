# futures-buffered

This project provides a single future structure: `FuturesUnorderedBounded`.

Much like [`futures::FuturesUnordered`](https://docs.rs/futures/0.3.25/futures/stream/struct.FuturesUnordered.html), this is a thread-safe, `Pin` friendly, lifetime friendly, concurrent processing stream.

The is different to `FuturesUnordered` in that `FuturesUnorderedBounded` has a fixed capacity for processing count. This means it's less flexible, but produces better memory efficiency.

## Benchmarks

### Speed

Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:

```
FuturesUnordered         time:   [420.47 ms 422.21 ms 423.99 ms]
FuturesUnorderedBounded  time:   [366.02 ms 367.54 ms 369.05 ms]
```

### Memory usage

Running 512000 `Ready<i32>` futures with 256 concurrent jobs.

- count: the number of times alloc/dealloc was called
- alloc: the number of cumulative bytes allocated
- dealloc: the number of cumulative bytes deallocated

```
FuturesUnordered
    count:    1024002
    alloc:    40960144 B
    dealloc:  40960000 B

FuturesUnorderedBounded
    count:    2
    alloc:    8264 B
    dealloc:  0 B
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
use futures::future::join_all;

async fn foo(i: u32) -> u32 { i }

let futures = vec![foo(1), foo(2), foo(3)];

assert_eq!(join_all(futures).await, [1, 2, 3]);
```
