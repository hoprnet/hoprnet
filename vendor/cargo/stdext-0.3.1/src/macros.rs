//! Various helper macros.

/// `compile_warning` macro is a brother of [`std::compile_error`],
/// which emits a compile-time warning with a provided message.
///
/// This implemented through an existing `dead_code` warning, thus the
/// output for the following example:
///
/// ```rust
/// # use stdext::compile_warning;
/// compile_warning!("Sample user-defined warning!");
/// ```
///
/// may look as follows:
///
/// ```text
/// warning: constant item is never used: `WARNING`
///   --> src/lib.rs:7:9
///   |
/// 7 |         const WARNING: &str = $expr;
///   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ...
/// 11 | compile_warning!("Sample user-defined warning!");
///    | ------------------------------------------------- in this macro invocation
/// ```
///
/// Once [`proc_macro_diagnostics`] feature is stabilized, this macro will be replaced
/// with a proper proc-macro-based implementation.
///
/// This macro is intended to be used in the development process, as an alternative to the
/// [`unimplemented`] macro which doesn't cause code to panic.
///
/// [`std::compile_error`]: https://doc.rust-lang.org/std/macro.compile_error.html
/// [`proc_macro_diagnostics`]: https://github.com/rust-lang/rust/issues/54140
/// [`unimplemented`]: https://doc.rust-lang.org/std/macro.unimplemented.html
#[macro_export]
macro_rules! compile_warning {
    ($expr:expr) => {
        #[warn(dead_code)]
        const WARNING: &str = $expr;
    };
}

/// This macro returns the name of the enclosing function.
/// As the internal implementation is based on the [`std::any::type_name`], this macro derives
/// all the limitations of this function.
///
/// ## Examples
///
/// ```rust
/// mod bar {
///     pub fn sample_function() {
///         use stdext::function_name;
///         assert!(function_name!().ends_with("bar::sample_function"));
///     }
/// }
///
/// bar::sample_function();
/// ```
///
/// [`std::any::type_name`]: https://doc.rust-lang.org/std/any/fn.type_name.html
#[macro_export]
macro_rules! function_name {
    () => {{
        // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // `3` is the length of the `::f`.
        &name[..name.len() - 3]
    }};
}

/// Attempts to get variant from the enum variable.
///
/// ## Examples
///
/// ```rust
/// # use stdext::try_match;
///
/// #[derive(Debug, PartialEq)]
/// enum Foo {
///     Left(u16),
///     Right(&'static str),
/// }
///
/// assert_eq!(try_match!(Foo::Left(18), Foo::Left), Ok(18));
/// assert_eq!(
///     try_match!(Foo::Right("nope"), Foo::Left),
///     Err(Foo::Right("nope"))
/// );
/// ```
#[macro_export]
macro_rules! try_match {
    ($var:expr, $variant:path) => {
        if let $variant(x) = $var {
            Ok(x)
        } else {
            Err($var)
        }
    };
}

/// Similar to [`try_match`] but additionally unwraps the result.
///
/// ## Panics
///
/// Panics if expression didn't match the provided path.
///
/// ## Examples
///
/// ```rust
/// # use stdext::unwrap_match;
///
/// #[derive(Debug, PartialEq)]
/// enum Foo {
///     Left(u16),
///     Right(&'static str),
/// }
///
/// assert_eq!(unwrap_match!(Foo::Left(18), Foo::Left), 18);
/// ```
///
/// The following example will panic:
///
/// ```should_panic
/// # use stdext::unwrap_match;
/// # #[derive(Debug, PartialEq)]
/// # enum Foo {
/// #     Left(u16),
/// #     Right(&'static str),
/// # }
/// assert_eq!(unwrap_match!(Foo::Right("nope"), Foo::Left), 18);
/// ```
#[macro_export]
macro_rules! unwrap_match {
    ($var:expr, $variant:path) => {
        $crate::try_match!($var, $variant).unwrap()
    };
}

/// Checks whether supplied [`Result`] variable is `Ok`
/// and if so, returns it.
///
/// If variant is an `Err`, macro evaluates to the contents of the `Err`
/// variant.
///
/// This macro supports two forms:
/// - `return_ok!(Ok(42));` - will return `Ok(42)`.
/// - `return_ok!(inner Ok(42));` - will return just `42`.
///
/// ## Examples
///
/// ```rust
/// # use stdext::return_ok;
///
/// fn choose_one(left: Result<u8, ()>, right: Result<u8, ()>) -> Result<u8, ()> {
///     return_ok!(left);
///     return_ok!(right);
///     Err(())
/// }
///
/// fn choose_one_inner(left: Result<u8, ()>, right: Result<u8, ()>) -> u8 {
///     return_ok!(inner left);
///     return_ok!(inner right);
///     panic!("Both variables are bad")
/// }
///
/// assert_eq!(choose_one(Err(()), Ok(10)), Ok(10));
/// assert_eq!(choose_one_inner(Ok(1), Err(())), 1);
/// ```
#[macro_export]
macro_rules! return_ok {
    ($var:expr) => {
        match $var {
            Ok(val) => return Ok(val),
            Err(err) => err,
        }
    };
    (inner $var:expr) => {
        match $var {
            Ok(val) => return val,
            Err(err) => err,
        }
    };
}

/// Checks whether supplied [`Option`] variable is `Some`
/// and if so, returns it.
///
/// If variant is an `None`, nothing happens.
///
/// This macro supports two forms:
/// - `return_some!(Some(42));` - will return `Some(42)`.
/// - `return_some!(inner Some(42));` - will return just `42`.
///
/// ## Examples
///
/// ```rust
/// # use stdext::return_some;
///
/// fn choose_one(left: Option<u8>, right: Option<u8>) -> Option<u8> {
///     return_some!(left);
///     return_some!(right);
///     None
/// }
///
/// fn choose_one_inner(left: Option<u8>, right: Option<u8>) -> u8 {
///     return_some!(inner left);
///     return_some!(inner right);
///     panic!("Both variables are bad")
/// }
///
/// assert_eq!(choose_one(None, Some(10)), Some(10));
/// assert_eq!(choose_one_inner(Some(1), None), 1);
/// ```
#[macro_export]
macro_rules! return_some {
    ($var:expr) => {
        match $var {
            Some(val) => return Some(val),
            None => {}
        }
    };
    (inner $var:expr) => {
        match $var {
            Some(val) => return val,
            None => {}
        }
    };
}
