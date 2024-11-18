//! # futures-buffered
//!
//! This project provides a single future structure: `FuturesUnorderedBounded`.
//!
//! Much like [`futures::stream::FuturesUnordered`](https://docs.rs/futures/0.3.25/futures/stream/struct.FuturesUnordered.html),
//! this is a thread-safe, `Pin` friendly, lifetime friendly, concurrent processing stream.
//!
//! The is different to `FuturesUnordered` in that `FuturesUnorderedBounded` has a fixed capacity for processing count.
//! This means it's less flexible, but produces better memory efficiency.
//!
//! ## Benchmarks
//!
//! ### Speed
//!
//! Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:
//!
//! ```text
//! FuturesUnordered         time:   [420.47 ms 422.21 ms 423.99 ms]
//! FuturesUnorderedBounded  time:   [366.02 ms 367.54 ms 369.05 ms]
//! ```
//!
//! ### Memory usage
//!
//! Running 512000 `Ready<i32>` futures with 256 concurrent jobs.
//!
//! - count: the number of times alloc/dealloc was called
//! - alloc: the number of cumulative bytes allocated
//! - dealloc: the number of cumulative bytes deallocated
//!
//! ```text
//! FuturesUnordered
//!     count:    1024002
//!     alloc:    40960144 B
//!     dealloc:  40960000 B
//!
//! FuturesUnorderedBounded
//!     count:    2
//!     alloc:    8264 B
//!     dealloc:  0 B
//! ```
//!
//! ### Conclusion
//!
//! As you can see, `FuturesUnorderedBounded` massively reduces you memory overhead while providing a significant performance gain.
//! Perfect for if you want a fixed batch size
//!
//! # Example
//! ```
//! use futures::future::Future;
//! use futures::stream::StreamExt;
//! use futures_buffered::FuturesUnorderedBounded;
//! use hyper::client::conn::http1::{handshake, SendRequest};
//! use hyper::body::Incoming;
//! use hyper::{Request, Response};
//! use hyper_util::rt::TokioIo;
//! use tokio::net::TcpStream;
//!
//! # #[cfg(miri)] fn main() {}
//! # #[cfg(not(miri))] #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // create a tcp connection
//! let stream = TcpStream::connect("example.com:80").await?;
//!
//! // perform the http handshakes
//! let (mut rs, conn) = handshake(TokioIo::new(stream)).await?;
//! tokio::spawn(conn);
//!
//! /// make http request to example.com and read the response
//! fn make_req(rs: &mut SendRequest<String>) -> impl Future<Output = hyper::Result<Response<Incoming>>> {
//!     let req = Request::builder()
//!         .header("Host", "example.com")
//!         .method("GET")
//!         .body(String::new())
//!         .unwrap();
//!     rs.send_request(req)
//! }
//!
//! // create a queue that can hold 128 concurrent requests
//! let mut queue = FuturesUnorderedBounded::new(128);
//!
//! // start up 128 requests
//! for _ in 0..128 {
//!     queue.push(make_req(&mut rs));
//! }
//! // wait for a request to finish and start another to fill its place - up to 1024 total requests
//! for _ in 128..1024 {
//!     queue.next().await;
//!     queue.push(make_req(&mut rs));
//! }
//! // wait for the tail end to finish
//! for _ in 0..128 {
//!     queue.next().await;
//! }
//! # Ok(()) }
//! ```
#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use(vec, dbg)]
extern crate std;

use core::future::Future;
use futures_core::Stream;

mod arc_slice;
mod buffered;
mod futures_ordered;
mod futures_ordered_bounded;
mod futures_unordered;
mod futures_unordered_bounded;
mod join_all;
mod merge_bounded;
mod merge_unbounded;
mod slot_map;
mod try_buffered;
mod try_join_all;

pub use buffered::{BufferUnordered, BufferedOrdered, BufferedStreamExt};
pub use futures_ordered::FuturesOrdered;
pub use futures_ordered_bounded::FuturesOrderedBounded;
pub use futures_unordered::FuturesUnordered;
pub use futures_unordered_bounded::FuturesUnorderedBounded;
pub use join_all::{join_all, JoinAll};
#[allow(deprecated)]
pub use merge_bounded::{Merge, MergeBounded};
pub use merge_unbounded::MergeUnbounded;
pub use try_buffered::{BufferedTryStreamExt, TryBufferUnordered, TryBufferedOrdered};
pub use try_join_all::{try_join_all, TryJoinAll};

mod private_try_future {
    use core::future::Future;

    pub trait Sealed {}

    impl<F, T, E> Sealed for F where F: ?Sized + Future<Output = Result<T, E>> {}
}

/// A convenience for futures that return `Result` values that includes
/// a variety of adapters tailored to such futures.
///
/// This is [`futures::TryFuture`](futures_core::future::TryFuture) except it's stricter on the future super-trait.
pub trait TryFuture:
    Future<Output = Result<Self::Ok, Self::Err>> + private_try_future::Sealed
{
    type Ok;
    type Err;
}

impl<T, E, F: ?Sized + Future<Output = Result<T, E>>> TryFuture for F {
    type Ok = T;
    type Err = E;
}

mod private_try_stream {
    use futures_core::Stream;

    pub trait Sealed {}

    impl<S, T, E> Sealed for S where S: ?Sized + Stream<Item = Result<T, E>> {}
}

/// A convenience for streams that return `Result` values that includes
/// a variety of adapters tailored to such futures.
///
/// This is [`futures::TryStream`](futures_core::stream::TryStream) except it's stricter on the stream super-trait.
pub trait TryStream:
    Stream<Item = Result<Self::Ok, Self::Err>> + private_try_stream::Sealed
{
    type Ok;
    type Err;
}

impl<T, E, S: ?Sized + Stream<Item = Result<T, E>>> TryStream for S {
    type Ok = T;
    type Err = E;
}
