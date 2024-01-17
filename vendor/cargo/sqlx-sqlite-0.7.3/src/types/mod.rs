//! Conversions between Rust and **SQLite** types.
//!
//! # Types
//!
//! | Rust type                             | SQLite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i8`                                  | INTEGER                                              |
//! | `i16`                                 | INTEGER                                              |
//! | `i32`                                 | INTEGER                                              |
//! | `i64`                                 | BIGINT, INT8                                         |
//! | `u8`                                  | INTEGER                                              |
//! | `u16`                                 | INTEGER                                              |
//! | `u32`                                 | INTEGER                                              |
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | REAL                                                 |
//! | `&str`, [`String`]                    | TEXT                                                 |
//! | `&[u8]`, `Vec<u8>`                    | BLOB                                                 |
//!
//! #### Note: Unsigned Integers
//! The unsigned integer types `u8`, `u16` and `u32` are implemented by zero-extending to the
//! next-larger signed type. So `u8` becomes `i16`, `u16` becomes `i32`, and `u32` becomes `i64`
//! while still retaining their semantic values.
//!
//! Similarly, decoding performs a checked truncation to ensure that overflow does not occur.
//!
//! SQLite stores integers in a variable-width encoding and always handles them in memory as 64-bit
//! signed values, so no space is wasted by this implicit widening.
//!
//! However, there is no corresponding larger type for `u64` in SQLite (it would require a `i128`),
//! and so it is not supported. Bit-casting it to `i64` or storing it as `REAL`, `BLOB` or `TEXT`
//! would change the semantics of the value in SQL and so violates the principle of least surprise.
//!
//! ### [`chrono`](https://crates.io/crates/chrono)
//!
//! Requires the `chrono` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `chrono::NaiveDateTime`               | DATETIME                                             |
//! | `chrono::DateTime<Utc>`               | DATETIME                                             |
//! | `chrono::DateTime<Local>`             | DATETIME                                             |
//! | `chrono::NaiveDate`                   | DATE                                                 |
//! | `chrono::NaiveTime`                   | TIME                                                 |
//!
//! ### [`time`](https://crates.io/crates/time)
//!
//! Requires the `time` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `time::PrimitiveDateTime`             | DATETIME                                             |
//! | `time::OffsetDateTime`                | DATETIME                                             |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//!
//! ### [`uuid`](https://crates.io/crates/uuid)
//!
//! Requires the `uuid` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | BLOB, TEXT                                           |
//! | `uuid::fmt::Hyphenated`               | TEXT                                                 |
//! | `uuid::fmt::Simple`                   | TEXT                                                 |
//!
//! ### [`json`](https://crates.io/crates/serde_json)
//!
//! Requires the `json` Cargo feature flag.
//!
//! | Rust type                             | Sqlite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | [`Json<T>`]                           | TEXT                                                 |
//! | `serde_json::JsonValue`               | TEXT                                                 |
//! | `&serde_json::value::RawValue`        | TEXT                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQLite.
//!
//! # Non-feature: `NUMERIC` / `rust_decimal` / `bigdecimal` Support
//! Support for mapping `rust_decimal::Decimal` and `bigdecimal::BigDecimal` to SQLite has been
//! deliberately omitted because SQLite does not have native support for high-
//! or arbitrary-precision decimal arithmetic, and to pretend so otherwise would be a
//! significant misstep in API design.
//!
//! The in-tree [`decimal.c`] extension is unfortunately not included in the [amalgamation],
//! which is used to build the bundled version of SQLite3 for `libsqlite3-sys` (which we have
//! enabled by default for the simpler setup experience), otherwise we could support that.
//!
//! The `NUMERIC` type affinity, while seemingly designed for storing decimal values,
//! stores non-integer real numbers as double-precision IEEE-754 floating point,
//! i.e. `REAL` in SQLite, `f64` in Rust, `double` in C/C++, etc.
//!
//! [Datatypes in SQLite: Type Affinity][type-affinity] (accessed 2023/11/20):
//!
//! > A column with NUMERIC affinity may contain values using all five storage classes.
//! When text data is inserted into a NUMERIC column, the storage class of the text is converted to
//! INTEGER or REAL (in order of preference) if the text is a well-formed integer or real literal,
//! respectively. If the TEXT value is a well-formed integer literal that is too large to fit in a
//! 64-bit signed integer, it is converted to REAL. For conversions between TEXT and REAL storage
//! classes, only the first 15 significant decimal digits of the number are preserved.
//!
//! With the SQLite3 interactive CLI, we can see that a higher-precision value
//! (20 digits in this case) is rounded off:
//!
//! ```text
//! sqlite> CREATE TABLE foo(bar NUMERIC);
//! sqlite> INSERT INTO foo(bar) VALUES('1.2345678901234567890');
//! sqlite> SELECT * FROM foo;
//! 1.23456789012346
//! ```
//!
//! It appears the `TEXT` storage class is only used if the value contains invalid characters
//! or extra whitespace.
//!
//! Thus, the `NUMERIC` type affinity is **unsuitable** for storage of high-precision decimal values
//! and should be **avoided at all costs**.
//!
//! Support for `rust_decimal` and `bigdecimal` would only be a trap because users will naturally
//! want to use the `NUMERIC` type affinity, and might otherwise encounter serious bugs caused by
//! rounding errors that they were deliberately avoiding when they chose an arbitrary-precision type
//! over a floating-point type in the first place.
//!
//! Instead, you should only use a type affinity that SQLite will not attempt to convert implicitly,
//! such as `TEXT` or `BLOB`, and map values to/from SQLite as strings. You can do this easily
//! using [the `Text` adapter].
//!
//!
//! [`decimal.c`]: https://www.sqlite.org/floatingpoint.html#the_decimal_c_extension
//! [amalgamation]: https://www.sqlite.org/amalgamation.html
//! [type-affinity]: https://www.sqlite.org/datatype3.html#type_affinity
//! [the `Text` adapter]: Text

pub(crate) use sqlx_core::types::*;

mod bool;
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
#[cfg(feature = "json")]
mod json;
mod str;
mod text;
#[cfg(feature = "time")]
mod time;
mod uint;
#[cfg(feature = "uuid")]
mod uuid;
