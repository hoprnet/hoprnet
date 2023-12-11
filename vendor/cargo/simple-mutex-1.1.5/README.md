# simple-mutex

[![Build](https://github.com/stjepang/simple-mutex/workflows/Build%20and%20test/badge.svg)](
https://github.com/stjepang/simple-mutex/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](
https://github.com/stjepang/simple-mutex)
[![Cargo](https://img.shields.io/crates/v/simple-mutex.svg)](
https://crates.io/crates/simple-mutex)
[![Documentation](https://docs.rs/simple-mutex/badge.svg)](
https://docs.rs/simple-mutex)

A simple mutex.

More efficient than
[`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
and simpler than
[`parking_lot::Mutex`](https://docs.rs/parking_lot).

The locking mechanism uses eventual fairness to ensure locking will be fair on average without
sacrificing performance. This is done by forcing a fair lock whenever a lock operation is
starved for longer than 0.5 milliseconds.

## Examples

```rust
use simple_mutex::Mutex;
use std::sync::Arc;
use std::thread;

let m = Arc::new(Mutex::new(0));
let mut threads = vec![];

for _ in 0..10 {
    let m = m.clone();
    threads.push(thread::spawn(move || {
        *m.lock() += 1;
    }));
}

for t in threads {
    t.join().unwrap();
}
assert_eq!(*m.lock(), 10);
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
