#![deny(missing_docs)]
//! This crate is for setting environment variables temporarily.
//!
//! It is useful for testing with different environment variables that should not interfere.
//!
//! # Examples
//!
//! ```rust
//! temp_env::with_var("MY_ENV_VAR", Some("production"), || {
//!     // Run some code where `MY_ENV_VAR` set to `"production"`.
//! });
//!
//! temp_env::with_vars(
//!     [
//!         ("FIRST_VAR", Some("Hello")),
//!         ("SECOND_VAR", Some("World!")),
//!     ],
//!     || {
//!         // Run some code where `FIRST_VAR` is set to `"Hello"` and `SECOND_VAR` is set to
//!         // `"World!"`.
//!     }
//! );
//!
//! temp_env::with_vars(
//!     [
//!         ("FIRST_VAR", Some("Hello")),
//!         ("SECOND_VAR", None),
//!     ],
//!     || {
//!         // Run some code where `FIRST_VAR` is set to `"Hello"` and `SECOND_VAR` is unset (even if
//!         // it was set before)
//!     }
//! );
//! ```
//!
//! It's possible the closure returns a value:
//!
//! ```rust
//! let s = temp_env::with_var("INNER_ENV_VAR", Some("inner value"), || {
//!      std::env::var("INNER_ENV_VAR").unwrap()
//! });
//! println!("{}", s);
//! ```
//!

use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::hash::Hash;

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

/// Make sure that the environment isn't modified concurrently.
static SERIAL_TEST: ReentrantMutex<()> = ReentrantMutex::new(());

/// Sets a single environment variable for the duration of the closure.
///
/// The previous value is restored when the closure completes or panics, before unwinding the
/// panic.
///
/// If `value` is set to `None`, then the environment variable is unset.
pub fn with_var<K, V, F, R>(key: K, value: Option<V>, closure: F) -> R
where
    K: AsRef<OsStr> + Clone + Eq + Hash,
    V: AsRef<OsStr> + Clone,
    F: FnOnce() -> R,
{
    with_vars([(key, value)], closure)
}

/// Unsets a single environment variable for the duration of the closure.
///
/// The previous value is restored when the closure completes or panics, before unwinding the
/// panic.
///
/// This is a shorthand and identical to the following:
/// ```rust
/// temp_env::with_var("MY_ENV_VAR", None::<&str>, || {
///     // Run some code where `MY_ENV_VAR` is unset.
/// });
/// ```
pub fn with_var_unset<K, F, R>(key: K, closure: F) -> R
where
    K: AsRef<OsStr> + Clone + Eq + Hash,
    F: FnOnce() -> R,
{
    with_var(key, None::<&str>, closure)
}

struct RestoreEnv<'a> {
    env: HashMap<&'a OsStr, Option<OsString>>,
    _guard: ReentrantMutexGuard<'a, ()>,
}

impl<'a> RestoreEnv<'a> {
    /// Capture the given variables from the environment.
    ///
    /// `guard` holds a lock on the shared mutex for exclusive access to the environment, to make
    /// sure that the environment gets restored while the lock is still held, i.e the current
    /// thread still has exclusive access to the environment.
    fn capture<I>(guard: ReentrantMutexGuard<'a, ()>, vars: I) -> Self
    where
        I: Iterator<Item = &'a OsStr> + 'a,
    {
        let env = vars.map(|v| (v, env::var_os(v))).collect();
        Self { env, _guard: guard }
    }
}

impl<'a> Drop for RestoreEnv<'a> {
    fn drop(&mut self) {
        for (var, value) in self.env.iter() {
            update_env(var, value.as_ref().map(|v| v.as_os_str()));
        }
    }
}

/// Sets environment variables for the duration of the closure.
///
/// The previous values are restored when the closure completes or panics, before unwinding the
/// panic.
///
/// If a `value` is set to `None`, then the environment variable is unset.
///
/// If the variable with the same name is set multiple times, the last one wins.
pub fn with_vars<K, V, F, R>(kvs: impl AsRef<[(K, Option<V>)]>, closure: F) -> R
where
    K: AsRef<OsStr> + Clone + Eq + Hash,
    V: AsRef<OsStr> + Clone,
    F: FnOnce() -> R,
{
    let old_env = RestoreEnv::capture(
        SERIAL_TEST.lock(),
        kvs.as_ref().iter().map(|(k, _)| k.as_ref()),
    );
    for (key, value) in kvs.as_ref() {
        update_env(key, value.as_ref());
    }
    let retval = closure();
    drop(old_env);
    retval
}

/// Unsets environment variables for the duration of the closure.
///
/// The previous values are restored when the closure completes or panics, before unwinding the
/// panic.
///
/// This is a shorthand and identical to the following:
/// ```rust
/// temp_env::with_vars(
///     [
///         ("FIRST_VAR", None::<&str>),
///         ("SECOND_VAR", None::<&str>),
///     ],
///     || {
///         // Run some code where `FIRST_VAR` and `SECOND_VAR` are unset (even if
///         // they were set before)
///     }
/// );
/// ```
pub fn with_vars_unset<K, F, R>(keys: impl AsRef<[K]>, closure: F) -> R
where
    K: AsRef<OsStr> + Clone + Eq + Hash,
    F: FnOnce() -> R,
{
    let kvs = keys
        .as_ref()
        .iter()
        .map(|key| (key, None::<&str>))
        .collect::<Vec<_>>();
    with_vars(kvs, closure)
}

fn update_env<K, V>(key: K, value: Option<V>)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    match value {
        Some(v) => env::set_var(key, v),
        None => env::remove_var(key),
    }
}

#[cfg(feature = "async_closure")]
/// Does the same as [`with_vars`] but it allows to pass an async closures.
///
/// ```rust
/// async fn check_var() {
///     let v = std::env::var("MY_VAR").unwrap();
///     assert_eq!(v, "ok".to_owned());
/// }

/// #[tokio::test]
/// async fn test_async_closure() {
///     crate::async_with_vars([("MY_VAR", Some("ok"))], check_var());
/// }
/// ```
pub async fn async_with_vars<K, V, F, R>(kvs: impl AsRef<[(K, Option<V>)]>, closure: F) -> R
where
    K: AsRef<OsStr> + Clone + Eq + Hash,
    V: AsRef<OsStr> + Clone,
    F: std::future::Future<Output = R> + std::future::IntoFuture<Output = R>,
{
    let old_env = RestoreEnv::capture(
        SERIAL_TEST.lock(),
        kvs.as_ref().iter().map(|(k, _)| k.as_ref()),
    );
    for (key, value) in kvs.as_ref() {
        update_env(key, value.as_ref());
    }
    let retval = closure.await;
    drop(old_env);
    retval
}

// Make sure that all tests use independent environment variables, so that they don't interfere if
// run in parallel.
#[cfg(test)]
mod tests {
    use std::env::VarError;
    use std::{env, panic};

    /// Test whether setting a variable is correctly undone.
    #[test]
    fn test_with_var_set() {
        let hello_not_set = env::var("HELLO");
        assert!(hello_not_set.is_err(), "`HELLO` must not be set.");

        crate::with_var("HELLO", Some("world!"), || {
            let hello_is_set = env::var("HELLO").unwrap();
            assert_eq!(hello_is_set, "world!", "`HELLO` must be set to \"world!\".");
        });

        let hello_not_set_after = env::var("HELLO");
        assert!(hello_not_set_after.is_err(), "`HELLO` must not be set.");
    }

    /// Test whether unsetting a variable is correctly undone.
    #[test]
    fn test_with_var_set_to_none() {
        env::set_var("FOO", "bar");
        let foo_is_set = env::var("FOO").unwrap();
        assert_eq!(foo_is_set, "bar", "`FOO` must be set to \"bar\".");

        crate::with_var("FOO", None::<&str>, || {
            let foo_not_set = env::var("FOO");
            assert!(foo_not_set.is_err(), "`FOO` must not be set.");
        });

        let foo_is_set_after = env::var("FOO").unwrap();
        assert_eq!(foo_is_set_after, "bar", "`FOO` must be set to \"bar\".");
    }

    /// Test whether unsetting a variable through the shorthand is correctly undone.
    #[test]
    fn test_with_var_unset() {
        env::set_var("BAR", "foo");
        let foo_is_set = env::var("BAR").unwrap();
        assert_eq!(foo_is_set, "foo", "`BAR` must be set to \"foo\".");

        crate::with_var_unset("BAR", || {
            let foo_not_set = env::var("BAR");
            assert!(foo_not_set.is_err(), "`BAR` must not be set.");
        });

        let foo_is_set_after = env::var("BAR").unwrap();
        assert_eq!(foo_is_set_after, "foo", "`BAR` must be set to \"foo\".");
    }

    /// Test whether overriding an existing variable is correctly undone.
    #[test]
    fn test_with_var_override() {
        env::set_var("BLAH", "blub");
        let blah_is_set = env::var("BLAH").unwrap();
        assert_eq!(blah_is_set, "blub", "`BLAH` must be set to \"blah\".");

        crate::with_var("BLAH", Some("new"), || {
            let blah_is_set_new = env::var("BLAH").unwrap();
            assert_eq!(blah_is_set_new, "new", "`BLAH` must be set to \"newb\".");
        });

        let blah_is_set_after = env::var("BLAH").unwrap();
        assert_eq!(
            blah_is_set_after, "blub",
            "`BLAH` must be set to \"blubr\"."
        );
    }

    /// Test whether overriding a variable is correctly undone in case of a panic.
    #[test]
    fn test_with_var_panic() {
        env::set_var("PANIC", "panic");
        let panic_is_set = env::var("PANIC").unwrap();
        assert_eq!(panic_is_set, "panic", "`PANIC` must be set to \"panic\".");

        let did_panic = panic::catch_unwind(|| {
            crate::with_var("PANIC", Some("don't panic"), || {
                let panic_is_set_new = env::var("PANIC").unwrap();
                assert_eq!(
                    panic_is_set_new, "don't panic",
                    "`PANIC` must be set to \"don't panic\"."
                );
                panic!("abort this closure with a panic.");
            });
        });

        assert!(did_panic.is_err(), "The closure must panic.");
        let panic_is_set_after = env::var("PANIC").unwrap();
        assert_eq!(
            panic_is_set_after, "panic",
            "`PANIC` must be set to \"panic\"."
        );
    }

    /// Test whether setting multiple variable is correctly undone.
    #[test]
    fn test_with_vars_set() {
        let one_not_set = env::var("ONE");
        assert!(one_not_set.is_err(), "`ONE` must not be set.");
        let two_not_set = env::var("TWO");
        assert!(two_not_set.is_err(), "`TWO` must not be set.");

        crate::with_vars([("ONE", Some("1")), ("TWO", Some("2"))], || {
            let one_is_set = env::var("ONE").unwrap();
            assert_eq!(one_is_set, "1", "`ONE` must be set to \"1\".");
            let two_is_set = env::var("TWO").unwrap();
            assert_eq!(two_is_set, "2", "`TWO` must be set to \"2\".");
        });

        let one_not_set_after = env::var("ONE");
        assert!(one_not_set_after.is_err(), "`ONE` must not be set.");
        let two_not_set_after = env::var("TWO");
        assert!(two_not_set_after.is_err(), "`TWO` must not be set.");
    }

    /// Test whether setting multiple variable is returns result.
    #[test]
    fn test_with_vars_set_returning() {
        let one_not_set = env::var("ONE");
        assert!(one_not_set.is_err(), "`ONE` must not be set.");
        let two_not_set = env::var("TWO");
        assert!(two_not_set.is_err(), "`TWO` must not be set.");

        let r = crate::with_vars([("ONE", Some("1")), ("TWO", Some("2"))], || {
            let one_is_set = env::var("ONE").unwrap();
            let two_is_set = env::var("TWO").unwrap();
            (one_is_set, two_is_set)
        });

        let (one_from_closure, two_from_closure) = r;

        assert_eq!(one_from_closure, "1", "`ONE` had to be set to \"1\".");
        assert_eq!(two_from_closure, "2", "`TWO` had to be set to \"2\".");

        let one_not_set_after = env::var("ONE");
        assert!(one_not_set_after.is_err(), "`ONE` must not be set.");
        let two_not_set_after = env::var("TWO");
        assert!(two_not_set_after.is_err(), "`TWO` must not be set.");
    }

    /// Test whether unsetting multiple variables is correctly undone.
    #[test]
    fn test_with_vars_unset() {
        env::set_var("SET_TO_BE_UNSET", "val");
        env::remove_var("UNSET_TO_BE_UNSET");
        // Check test preconditions
        assert_eq!(env::var("SET_TO_BE_UNSET"), Ok("val".to_string()));
        assert_eq!(env::var("UNSET_TO_BE_UNSET"), Err(VarError::NotPresent));

        crate::with_vars_unset(["SET_TO_BE_UNSET", "UNSET_TO_BE_UNSET"], || {
            assert_eq!(env::var("SET_TO_BE_UNSET"), Err(VarError::NotPresent));
            assert_eq!(env::var("UNSET_TO_BE_UNSET"), Err(VarError::NotPresent));
        });
        assert_eq!(env::var("SET_TO_BE_UNSET"), Ok("val".to_string()));
        assert_eq!(env::var("UNSET_TO_BE_UNSET"), Err(VarError::NotPresent));
    }

    /// Test whether unsetting one of the variable is correctly undone.
    #[test]
    fn test_with_vars_partially_unset() {
        let to_be_set_not_set = env::var("TO_BE_SET");
        assert!(to_be_set_not_set.is_err(), "`TO_BE_SET` must not be set.");
        env::set_var("TO_BE_UNSET", "unset");
        let to_be_unset_is_set = env::var("TO_BE_UNSET").unwrap();
        assert_eq!(
            to_be_unset_is_set, "unset",
            "`TO_BE_UNSET` must be set to \"unset\"."
        );

        crate::with_vars(
            [("TO_BE_SET", Some("set")), ("TO_BE_UNSET", None::<&str>)],
            || {
                let to_be_set_is_set = env::var("TO_BE_SET").unwrap();
                assert_eq!(
                    to_be_set_is_set, "set",
                    "`TO_BE_SET` must be set to \"set\"."
                );
                let to_be_unset_not_set = env::var("TO_BE_UNSET");
                assert!(
                    to_be_unset_not_set.is_err(),
                    "`TO_BE_UNSET` must not be set."
                );
            },
        );

        let to_be_set_not_set_after = env::var("TO_BE_SET");
        assert!(
            to_be_set_not_set_after.is_err(),
            "`TO_BE_SET` must not be set."
        );
        let to_be_unset_is_set_after = env::var("TO_BE_UNSET").unwrap();
        assert_eq!(
            to_be_unset_is_set_after, "unset",
            "`TO_BE_UNSET` must be set to \"unset\"."
        );
    }

    /// Test whether overriding existing variables is correctly undone.
    #[test]
    fn test_with_vars_override() {
        env::set_var("DOIT", "doit");
        let doit_is_set = env::var("DOIT").unwrap();
        assert_eq!(doit_is_set, "doit", "`DOIT` must be set to \"doit\".");
        env::set_var("NOW", "now");
        let now_is_set = env::var("NOW").unwrap();
        assert_eq!(now_is_set, "now", "`NOW` must be set to \"now\".");

        crate::with_vars([("DOIT", Some("other")), ("NOW", Some("value"))], || {
            let doit_is_set_new = env::var("DOIT").unwrap();
            assert_eq!(doit_is_set_new, "other", "`DOIT` must be set to \"other\".");
            let now_is_set_new = env::var("NOW").unwrap();
            assert_eq!(now_is_set_new, "value", "`NOW` must be set to \"value\".");
        });

        let doit_is_set_after = env::var("DOIT").unwrap();
        assert_eq!(doit_is_set_after, "doit", "`DOIT` must be set to \"doit\".");
        let now_is_set_after = env::var("NOW").unwrap();
        assert_eq!(now_is_set_after, "now", "`NOW` must be set to \"now\".");
    }

    /// Test that setting the same variables twice, the latter one is used.
    #[test]
    fn test_with_vars_same_vars() {
        let override_not_set = env::var("OVERRIDE");
        assert!(override_not_set.is_err(), "`OVERRIDE` must not be set.");

        crate::with_vars(
            [
                ("OVERRIDE", Some("initial")),
                ("OVERRIDE", Some("override")),
            ],
            || {
                let override_is_set = env::var("OVERRIDE").unwrap();
                assert_eq!(
                    override_is_set, "override",
                    "`OVERRIDE` must be set to \"override\"."
                );
            },
        );

        let override_not_set_after = env::var("OVERRIDE");
        assert!(
            override_not_set_after.is_err(),
            "`OVERRIDE` must not be set."
        );
    }

    /// Test that unsetting and setting the same variable leads to the variable being set.
    #[test]
    fn test_with_vars_unset_set() {
        env::set_var("MY_VAR", "my_var");
        let my_var_is_set = env::var("MY_VAR").unwrap();
        assert_eq!(
            my_var_is_set, "my_var",
            "`MY_VAR` must be set to \"my_var`\"."
        );

        crate::with_vars(
            [("MY_VAR", None::<&str>), ("MY_VAR", Some("new value"))],
            || {
                let my_var_is_set_new = env::var("MY_VAR").unwrap();
                assert_eq!(
                    my_var_is_set_new, "new value",
                    "`MY_VAR` must be set to \"new value\"."
                );
            },
        );

        let my_var_is_set_after = env::var("MY_VAR").unwrap();
        assert_eq!(
            my_var_is_set_after, "my_var",
            "`MY_VAR` must be set to \"my_var\"."
        );
    }

    /// Test that setting and unsetting the same variable leads to the variable being unset.
    #[test]
    fn test_with_vars_set_unset() {
        let not_my_var_not_set = env::var("NOT_MY_VAR");
        assert!(not_my_var_not_set.is_err(), "`NOT_MY_VAR` must not be set.");

        crate::with_vars(
            [
                ("NOT_MY_VAR", Some("it is set")),
                ("NOT_MY_VAR", None::<&str>),
            ],
            || {
                let not_my_var_not_set_new = env::var("NOT_MY_VAR");
                assert!(
                    not_my_var_not_set_new.is_err(),
                    "`NOT_MY_VAR` must not be set."
                );
            },
        );

        let not_my_var_not_set_after = env::var("NOT_MY_VAR");
        assert!(
            not_my_var_not_set_after.is_err(),
            "`NOT_MY_VAR` must not be set."
        );
    }

    #[test]
    fn test_with_nested_set() {
        crate::with_var("MY_VAR_1", Some("1"), || {
            crate::with_var("MY_VAR_2", Some("2"), || {
                assert_eq!(env::var("MY_VAR_1").unwrap(), "1");
                assert_eq!(env::var("MY_VAR_2").unwrap(), "2");
            })
        });

        assert!(env::var("MY_VAR_1").is_err());
        assert!(env::var("MY_VAR_2").is_err());
    }

    #[test]
    fn test_fn_once() {
        let value = String::from("Hello, ");
        let value = crate::with_var("WORLD", Some("world!"), || {
            value + &env::var("WORLD").unwrap()
        });
        assert_eq!(value, "Hello, world!");
    }

    #[cfg(feature = "async_closure")]
    async fn check_var() {
        let v = std::env::var("MY_VAR").unwrap();
        assert_eq!(v, "ok".to_owned());
    }

    #[cfg(feature = "async_closure")]
    #[tokio::test]
    async fn test_async_closure() {
        crate::async_with_vars([("MY_VAR", Some("ok"))], check_var()).await;
        let f = async {
            let v = std::env::var("MY_VAR").unwrap();
            assert_eq!(v, "ok".to_owned());
        };
        crate::async_with_vars([("MY_VAR", Some("ok"))], f).await;
    }

    #[cfg(feature = "async_closure")]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_closure_calls_closure() {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let f = async {
            tx.send(std::env::var("MY_VAR")).unwrap();
        };
        crate::async_with_vars([("MY_VAR", Some("ok"))], f).await;
        let value = rx.await.unwrap().unwrap();
        assert_eq!(value, "ok".to_owned());
    }

    #[cfg(feature = "async_closure")]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_with_vars_set_returning() {
        let one_not_set = env::var("ONE");
        assert!(one_not_set.is_err(), "`ONE` must not be set.");
        let two_not_set = env::var("TWO");
        assert!(two_not_set.is_err(), "`TWO` must not be set.");

        let r = crate::async_with_vars([("ONE", Some("1")), ("TWO", Some("2"))], async {
            let one_is_set = env::var("ONE").unwrap();
            let two_is_set = env::var("TWO").unwrap();
            (one_is_set, two_is_set)
        })
        .await;

        let (one_from_closure, two_from_closure) = r;

        assert_eq!(one_from_closure, "1", "`ONE` had to be set to \"1\".");
        assert_eq!(two_from_closure, "2", "`TWO` had to be set to \"2\".");

        let one_not_set_after = env::var("ONE");
        assert!(one_not_set_after.is_err(), "`ONE` must not be set.");
        let two_not_set_after = env::var("TWO");
        assert!(two_not_set_after.is_err(), "`TWO` must not be set.");
    }
}
