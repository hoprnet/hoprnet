//! # tracing_test_macro
//!
//! This crate provides a procedural macro that can be added to test functions in order to ensure
//! that all tracing logs are written to a global buffer.
//!
//! You should not use this crate directly. Instead, use the macro through [tracing-test].
//!
//! [tracing-test]: https://docs.rs/tracing-test
extern crate proc_macro;

use std::sync::Mutex;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, ItemFn, Stmt};

lazy_static! {
    /// Registered scopes.
    ///
    /// By default, every traced test registers a span with the function name.
    /// However, since multiple tests can share the same function name, in case
    /// of conflict, a counter is appended.
    ///
    /// This vector is used to store all already registered scopes.
    static ref REGISTERED_SCOPES: Mutex<Vec<String>> = Mutex::new(vec![]);
}

/// Check whether this test function name is already taken as scope. If yes, a
/// counter is appended to make it unique. In the end, a unique scope is returned.
fn get_free_scope(mut test_fn_name: String) -> String {
    let mut vec = REGISTERED_SCOPES.lock().unwrap();
    let mut counter = 1;
    let len = test_fn_name.len();
    while vec.contains(&test_fn_name) {
        counter += 1;
        test_fn_name.replace_range(len.., &counter.to_string());
    }
    vec.push(test_fn_name.clone());
    test_fn_name
}

/// A procedural macro that ensures that a global logger is registered for the
/// annotated test.
///
/// Additionally, the macro injects a local function called `logs_contain`,
/// which can be used to assert that a certain string was logged within this
/// test.
///
/// Check out the docs of the `tracing-test` crate for more usage information.
#[proc_macro_attribute]
pub fn traced_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse annotated function
    let mut function: ItemFn = parse(item).expect("Could not parse ItemFn");

    // Determine scope
    let scope = get_free_scope(function.sig.ident.to_string());

    // Determine features
    //
    // Note: This cannot be called in the block below, otherwise it would be
    //       evaluated in the context of the calling crate, not of the macro
    //       crate!
    let no_env_filter = cfg!(feature = "no-env-filter");

    // Prepare code that should be injected at the start of the function
    let init = parse::<Stmt>(
        quote! {
            tracing_test::internal::INITIALIZED.call_once(|| {
                let env_filter = if #no_env_filter {
                    "trace".to_string()
                } else {
                    let crate_name = module_path!()
                        .split(":")
                        .next()
                        .expect("Could not find crate name in module path")
                        .to_string();
                    format!("{}=trace", crate_name)
                };
                let mock_writer = tracing_test::internal::MockWriter::new(&tracing_test::internal::GLOBAL_BUF);
                let subscriber = tracing_test::internal::get_subscriber(mock_writer, &env_filter);
                tracing::dispatcher::set_global_default(subscriber)
                    .expect("Could not set global tracing subscriber");
            });
        }
        .into(),
    )
    .expect("Could not parse quoted statement init");
    let span = parse::<Stmt>(
        quote! {
            let span = tracing::info_span!(#scope);
        }
        .into(),
    )
    .expect("Could not parse quoted statement span");
    let enter = parse::<Stmt>(
        quote! {
            let _enter = span.enter();
        }
        .into(),
    )
    .expect("Could not parse quoted statement enter");
    let logs_contain_fn = parse::<Stmt>(
        quote! {
            fn logs_contain(val: &str) -> bool {
                tracing_test::internal::logs_with_scope_contain(#scope, val)
            }

        }
        .into(),
    )
    .expect("Could not parse quoted statement logs_contain_fn");
    let logs_assert_fn = parse::<Stmt>(
        quote! {
            /// Run a function against the log lines. If the function returns
            /// an `Err`, panic. This can be used to run arbitrary assertion
            /// logic against the logs.
            fn logs_assert(f: impl Fn(&[&str]) -> std::result::Result<(), String>) {
                match tracing_test::internal::logs_assert(#scope, f) {
                    Ok(()) => {},
                    Err(msg) => panic!("The logs_assert function returned an error: {}", msg),
                };
            }
        }
        .into(),
    )
    .expect("Could not parse quoted statement logs_assert_fn");

    // Inject code into function
    function.block.stmts.insert(0, init);
    function.block.stmts.insert(1, span);
    function.block.stmts.insert(2, enter);
    function.block.stmts.insert(3, logs_contain_fn);
    function.block.stmts.insert(4, logs_assert_fn);

    // Generate token stream
    TokenStream::from(function.to_token_stream())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_free_scope() {
        let initial = get_free_scope("test_fn_name".to_string());
        assert_eq!(initial, "test_fn_name");

        let second = get_free_scope("test_fn_name".to_string());
        assert_eq!(second, "test_fn_name2");
        let third = get_free_scope("test_fn_name".to_string());
        assert_eq!(third, "test_fn_name3");

        // Insert a conflicting entry
        let fourth = get_free_scope("test_fn_name4".to_string());
        assert_eq!(fourth, "test_fn_name4");

        let fifth = get_free_scope("test_fn_name5".to_string());
        assert_eq!(fifth, "test_fn_name5");
    }
}
