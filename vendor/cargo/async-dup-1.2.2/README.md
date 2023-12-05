# async-dup

[![Build](https://github.com/stjepang/async-dup/workflows/Build%20and%20test/badge.svg)](
https://github.com/stjepang/async-dup/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/stjepang/async-dup)
[![Cargo](https://img.shields.io/crates/v/async-dup.svg)](
https://crates.io/crates/async-dup)
[![Documentation](https://docs.rs/async-dup/badge.svg)](
https://docs.rs/async-dup)

Duplicate an async I/O handle.

This crate provides two tools, `Arc` and `Mutex`:

* `Arc` implements `AsyncRead`, `AsyncWrite`, and `AsyncSeek` if a reference to the inner type does.
* A reference to `Mutex` implements `AsyncRead`, `AsyncWrite`, and `AsyncSeek` if the inner type does.

Wrap an async I/O handle in `Arc` or `Mutex` to clone it or share among tasks.

## Examples

Clone an async I/O handle:

```rust
use async_dup::Arc;
use futures::io;
use smol::Async;
use std::net::TcpStream;

// A client that echoes messages back to the server.
let stream = Async::<TcpStream>::connect("127.0.0.1:8000").await?;

// Create two handles to the stream.
let reader = Arc::new(stream);
let mut writer = reader.clone();

// Echo data received from the reader back into the writer.
io::copy(reader, &mut writer).await?;
```

Share an async I/O handle:

```rust
use async_dup::Mutex;
use futures::io;
use futures::prelude::*;

// Reads data from a stream and echoes it back.
async fn echo(stream: impl AsyncRead + AsyncWrite + Unpin) -> io::Result<u64> {
    let stream = Mutex::new(stream);
    io::copy(&stream, &mut &stream).await
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
