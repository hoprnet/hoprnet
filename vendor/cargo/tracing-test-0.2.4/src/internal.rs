//! Internal functionality used by the [`#[traced_test]`](attr.traced_test.html) macro.
//!
//! These functions should usually not be accessed from user code. The stability of these functions
//! is not guaranteed, the API may change even in patch releases.
use std::sync::{Mutex, Once};

use lazy_static::lazy_static;

pub use crate::subscriber::{get_subscriber, MockWriter};

/// Static variable to ensure that logging is only initialized once.
pub static INITIALIZED: Once = Once::new();

lazy_static! {
    /// The global log output buffer used in tests.
    #[doc(hidden)]
    pub static ref GLOBAL_BUF: Mutex<Vec<u8>> = Mutex::new(vec![]);
}

/// Return whether the logs with the specified scope contain the specified value.
///
/// This function should usually not be used directly, instead use the `logs_contain(val: &str)`
/// function injected by the [`#[traced_test]`](attr.traced_test.html) macro.
pub fn logs_with_scope_contain(scope: &str, val: &str) -> bool {
    let logs = String::from_utf8(GLOBAL_BUF.lock().unwrap().to_vec()).unwrap();
    for line in logs.split('\n') {
        if line.contains(&format!(" {}:", scope)) && line.contains(val) {
            return true;
        }
    }
    false
}

/// Run a function against a slice of logs for the specified scope and return
/// its result.
///
/// This function should usually not be used directly, instead use the
/// `logs_assert(F) where F: Fn(&[&str]) -> Result<(), String>` function
/// injected by the [`#[traced_test]`](attr.traced_test.html) macro.
pub fn logs_assert<F>(scope: &str, f: F) -> std::result::Result<(), String>
where
    F: Fn(&[&str]) -> std::result::Result<(), String>,
{
    let buf = GLOBAL_BUF.lock().unwrap();
    let logs: Vec<&str> = std::str::from_utf8(&buf)
        .expect("Logs contain invalid UTF8")
        .lines()
        .filter(|line| line.contains(&format!(" {}:", scope)))
        .collect();
    f(&logs)
}
