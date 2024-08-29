# tokio-retry

Extensible, asynchronous retry behaviours for the ecosystem of [tokio](https://tokio.rs/) libraries.

[![Build Status](https://travis-ci.org/srijs/rust-tokio-retry.svg?branch=master)](https://travis-ci.org/srijs/rust-tokio-retry)
[![crates](http://meritbadge.herokuapp.com/tokio-retry)](https://crates.io/crates/tokio-retry)
[![dependency status](https://deps.rs/repo/github/srijs/rust-tokio-retry/status.svg)](https://deps.rs/repo/github/srijs/rust-tokio-retry)


[Documentation](https://docs.rs/tokio-retry)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokio-retry = "0.3"
```

## Examples

```rust
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

async fn action() -> Result<u64, ()> {
    // do some real-world stuff here...
    Err(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // add jitter to delays
        .take(3);    // limit to 3 retries

    let result = Retry::spawn(retry_strategy, action).await?;

    Ok(())
}
```
