//! Helper functions and macros that allow for easier testing of crates that use `tracing`.
//!
//! The focus is on testing the logging, not on debugging the tests. That's why the
//! library ensures that the logs do not depend on external state. For example, the
//! `RUST_LOG` env variable is not used for log filtering.
//!
//! Similar crates:
//!
//! - [test-log](https://crates.io/crates/test-log): Initialize loggers before
//!   running tests
//! - [tracing-fluent-assertions](https://crates.io/crates/tracing-fluent-assertions):
//!   More powerful assertions that also allow analyzing spans
//!
//! ## Usage
//!
//! This crate should mainly be used through the
//! [`#[traced_test]`](attr.traced_test.html) macro.
//!
//! First, add a dependency on `tracing-test` in `Cargo.toml`:
//!
//! ```toml
//! tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
//! tracing = "0.1"
//! tracing-test = "0.1"
//! ```
//!
//! Then, annotate your test function with the `#[traced_test]` macro.
//!
//! ```rust
//! use tracing::{info, warn};
//! use tracing_test::traced_test;
//!
//! #[tokio::test]
//! #[traced_test]
//! async fn test_logs_are_captured() {
//!     // Local log
//!     info!("This is being logged on the info level");
//!
//!     // Log from a spawned task (which runs in a separate thread)
//!     tokio::spawn(async {
//!         warn!("This is being logged on the warn level from a spawned task");
//!     })
//!     .await
//!     .unwrap();
//!
//!     // Ensure that certain strings are or aren't logged
//!     assert!(logs_contain("logged on the info level"));
//!     assert!(logs_contain("logged on the warn level"));
//!     assert!(!logs_contain("logged on the error level"));
//!
//!     // Ensure that the string `logged` is logged exactly twice
//!     logs_assert(|lines: &[&str]| {
//!         match lines.iter().filter(|line| line.contains("logged")).count() {
//!             2 => Ok(()),
//!             n => Err(format!("Expected two matching logs, but found {}", n)),
//!         }
//!     });
//! }
//! ```
//!
//! Done! You can write assertions using one of two injected functions:
//!
//! - `logs_contain(&str) -> bool`: Use this within an `assert!` call to ensure
//!   that a certain string is (or isn't) logged anywhere in the logs.
//! - `logs_assert(f: impl Fn(&[&str]) -> Result<(), String>)`:  Run a function
//!   against the log lines. If the function returns an `Err`, panic. This can
//!   be used to run arbitrary assertion logic against the logs.
//!
//! Logs are written to stdout, so they are captured by the cargo test runner
//! by default, but printed if the test fails.
//!
//! Of course, you can also annotate regular non-async tests:
//!
//! ```rust
//! use tracing::info;
//! use tracing_test::traced_test;
//!
//! #[traced_test]
//! #[test]
//! fn plain_old_test() {
//!     assert!(!logs_contain("Logging from a non-async test"));
//!     info!("Logging from a non-async test");
//!     assert!(logs_contain("Logging from a non-async test"));
//!     assert!(!logs_contain("This was never logged"));
//! }
//! ```
//!
//! ## Rationale / Why You Need This
//!
//! Tracing allows you to set a default subscriber within a scope:
//!
//! ```rust
//! # let subscriber = tracing::Dispatch::new(tracing_subscriber::FmtSubscriber::new());
//! # let req = 123;
//! # fn get_response(fake_req: u8) {}
//! let response = tracing::dispatcher::with_default(&subscriber, || get_response(req));
//! ```
//!
//! This works fine, as long as no threads are involved. As soon as you use a
//! multi-threaded test runtime (e.g. the `#[tokio::test]` with the
//! `rt-multi-thread` feature) and spawn tasks, the tracing logs in those tasks
//! will not be captured by the subscriber.
//!
//! The macro provided in this crate registers a global default subscriber instead.
//! This subscriber contains a writer which logs into a global static in-memory buffer.
//!
//! At the beginning of every test, the macro injects span opening code. The span
//! uses the name of the test function (unless it's already taken, then a counter
//! is appended). This means that the logs from a test are prefixed with the test
//! name, which helps when debugging.
//!
//! Finally, a function called `logs_contain(value: &str)` is injected into every
//! annotated test. It filters the logs in the buffer to include only lines
//! containing ` {span_name}: ` and then searches the value in the matching log
//! lines. This can be used to assert that a message was logged during a test.
//!
//! ## Per-crate Filtering
//!
//! By default, `tracing-test` sets an env filter that filters out all logs
//! except the ones from your crate (equivalent to
//! `RUST_LOG=<your_crate>=trace`). If you need to capture logs from other crates
//! as well, you can turn off this log filtering globally by enabling the
//! `no-env-filter` Cargo feature:
//!
//! ```toml
//! tracing-test = { version = "0.1", features = ["no-env-filter"] }
//! ```
//!
//! Note that this will result in _all_ logs from _all_ your dependencies being
//! captured! This means that the `logs_contain` function may become less
//! useful, and you might need to use `logs_assert` instead, with your own
//! custom filtering logic.
//!
//! **Note:** Rust "integration tests" (in the `tests/` directory) are each
//! built into a separate crate from the crate they test. As a result,
//! integration tests must use `no-env-filter` to capture and observe logs.

pub mod internal;
mod subscriber;

pub use tracing_test_macro::traced_test;
