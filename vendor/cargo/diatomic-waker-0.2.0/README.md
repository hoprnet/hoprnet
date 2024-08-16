# diatomic-waker

An async, fast synchronization primitives for task wakeup.

[![Cargo](https://img.shields.io/crates/v/diatomic-waker.svg)](https://crates.io/crates/diatomic-waker)
[![Documentation](https://docs.rs/diatomic-waker/badge.svg)](https://docs.rs/diatomic-waker)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/asynchronics/diatomic-waker#license)

## Overview

`diatomic-waker` is similar to [`atomic-waker`][atomic-waker] in that it enables
concurrent updates and notifications to a wrapped `Waker`. Unlike the latter,
however, it does not use spinlocks[^spinlocks] and is significantly faster, in
particular when the consumer needs to be notified periodically rather than just
once. It can in particular be used as a very fast, single-consumer [eventcount]
to turn a non-blocking data structure into an asynchronous one (see MPSC channel
receiver example).

This library is an offshoot of [Asynchronix][asynchronix], an ongoing effort at
a high performance asynchronous computation framework for system simulation.

[atomic-waker]: https://docs.rs/atomic-waker/latest/atomic_waker/
[eventcount]: https://www.1024cores.net/home/lock-free-algorithms/eventcounts
[asynchronix]: https://github.com/asynchronics/asynchronix
[^spinlocks]: The implementation of [AtomicWaker][atomic-waker] yields to the
    runtime on contention, which is in effect an executor-mediated spinlock.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
diatomic-waker = "0.2.0"
```

## Features flags

By default, this crate enables the `alloc` feature to provide the owned
`WakeSink` and `WakeSource`. It can be made `no-std`-compatible by specifying
`default-features = false`.

## Example

A multi-producer, single-consumer channel of capacity 1 for sending
`NonZeroUsize` values, with an asynchronous receiver:

```rust
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use diatomic_waker::{WakeSink, WakeSource};

// The sending side of the channel.
#[derive(Clone)]
struct Sender {
    wake_src: WakeSource,
    value: Arc<AtomicUsize>,
}

// The receiving side of the channel.
struct Receiver {
    wake_sink: WakeSink,
    value: Arc<AtomicUsize>,
}

// Creates an empty channel.
fn channel() -> (Sender, Receiver) {
    let value = Arc::new(AtomicUsize::new(0));
    let wake_sink = WakeSink::new();
    let wake_src = wake_sink.source();
    (
        Sender {
            wake_src,
            value: value.clone(),
        },
        Receiver { wake_sink, value },
    )
}

impl Sender {
    // Sends a value if the channel is empty.
    fn try_send(&self, value: NonZeroUsize) -> bool {
        let success = self
            .value
            .compare_exchange(0, value.get(), Ordering::Relaxed, Ordering::Relaxed)
            .is_ok();
        if success {
            self.wake_src.notify()
        };
        success
    }
}

impl Receiver {
    // Receives a value asynchronously.
    async fn recv(&mut self) -> NonZeroUsize {
        // Wait until the predicate returns `Some(value)`, i.e. when the atomic
        // value becomes non-zero.
        self.wake_sink
            .wait_until(|| NonZeroUsize::new(self.value.swap(0, Ordering::Relaxed)))
            .await
    }
}
```

## Safety

This is a low-level primitive and as such its implementation relies on `unsafe`.
The test suite makes extensive use of [Loom] to assess its correctness. As
amazing as it is, however, Loom is only a tool: it cannot formally prove the
absence of data races.

[Loom]: https://github.com/tokio-rs/loom


## Implementation details

A distinguishing feature of `diatomic-waker` is its use of two waker storage
slots (hence its name) rather than one. This makes it possible to achieve
lock-freedom in situations where waker registration and notification are
performed concurrently. In the case of concurrent notifications, even though one
notifier does hold a notification lock, other notifiers never block: they merely
request the holder of the lock to send another notification, which is a
wait-free operation.

Compared to `atomic-waker`, dummy notifications (with no waker registered) are
much cheaper. The overall cost of a successful notification (registration +
notification itself) is also much cheaper in the common case where the
registered/unregistered waker is always the same, because the last waker is
always cached to avoid undue cloning. Quantitatively, the costs in terms of
atomic Read-Modify-Write (RMW) operations are:

* dummy notification due to no waker being registered: 1 RMW vs 2 RMWs for
  `atomic-waker`,
* registration of the same waker as the last registered waker + notification:
  1+3=4 RMWs vs 3+4=7 RMWs for `atomic-waker` (this assumes 1 RMW for
  `Waker::wake_by_ref`, 2 RMWs for `Waker::wake`, 1 RMW for `Waker::clone`).
* registration of a new waker + notification: 3+3=6 RMWs vs 3+4=7 RMWs for
  `atomic-waker` (same assumptions as above + 1 RMW for `Waker::drop`); this is
  typically only necessary for the very first registration,
* very few RMWs and predictable cost on contention due to the absence of
  spinlocks.


## License

This software is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT), at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
