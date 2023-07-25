//! Additional features for the Rust standard library.
//!
//! ## Description
//!
//! This crate contains enhancements to the Rust standard library types, useful for
//! broad audience, but not yet implemented (or stabilized) in `std`.
//!
//! Crate is designed to be lightweight (no external dependencies!) and provide essential
//! functionality which possible can get to the `std` some day.
//!
//! ## Extension traits
//!
//! All the new functionality the stanard library is added using extension traits.
//!
//! Below you can find the table of all the extension traits introduced by this crate:
//!
//! | `std` structure | extension traits
//! | --- | ---
//! | [`Vec`] | [`VecExt`] and [`VecExtClone`]
//! | [`&str`] | [`StrExt`]
//! | [`Option`] | [`OptionExt`]
//! | [`Result`] | [`ResultExt`]
//! | [`Duration`] | [`DurationExt`]
//! | [`RwLock`] | [`RwLockExt`]
//! | [`Mutex`] | [`MutexExt`]
//! | [`f32`] and [`f64`] | [`FloatConvert`]
//!
//! [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
//! [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
//! [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
//! [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
//! [`Duration`]: https://doc.rust-lang.org/std/time/struct.Duration.html
//! [`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
//! [`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
//!
//! [`VecExt`]: vec/trait.VecExt.html
//! [`VecExtClone`]: vec/trait.VecExtClone.html
//! [`StrExt`]: str/trait.StrExt.html
//! [`OptionExt`]: option/trait.OptionExt.html
//! [`ResultExt`]: result/trait.ResultExt.html
//! [`DurationExt`]: duration/trait.DurationExt.html
//! [`RwLockExt`]: sync/rw_lock/trait.RwLockExt.html
//! [`MutexExt`]: sync/mutex/trait.MutexExt.html
//! [`FloatConvert`]: num/float_convert/trait.FloatConvert.html
//!
//! ## Integer super-trait
//!
//! While all built-in integer types have mostly the same interface, it's not backed by any trait,
//! which makes it impossible to write a function that will accept any built-in integer.
//!
//! [`Integer`] trait solves that problem by reflecting the common interface of all the built-in integers.
//!
//! [`Integer`]: num/integer/trait.Integer.html
//!
//! ## Macros
//!
//! Another group of extensions in `stdext` is new macros:
//!
//! - [`compile_warning`] for spawning a user-defined compilation warnings.
//! - [`function_name`] for getting an enclosing function name.
//!
//! [`compile_warning`]: ./macro.compile_warning.html
//! [`function_name`]: ./macro.function_name.html
//!
//! ## Highlights
//!
//! - Convenient builder methods for **`Duration`**:
//!   
//!   ```rust
//!   use std::time::Duration;
//!   use stdext::prelude::*;
//!
//!   let duration = Duration::from_minutes(1).add_secs(5).add_micros(100);
//!   assert_eq!(duration, Duration::new(65, 100_000));
//!   ```
//!
//! - Panicking version for **`RwLock::read`**, **`RwLock::write`** and **`Mutex::lock`** (for
//!   fellows who don't really handle the lock poisoning):
//!
//!   ```rust
//!   use std::sync::{Arc, RwLock};
//!   use stdext::prelude::*;
//!   
//!   let lock = Arc::new(RwLock::new(1));
//!   {
//!       let mut n = lock.force_write(); // Instead of `.write().unwrap()`.
//!       *n = 2;
//!   }
//!   let n = lock.force_read();
//!   assert_eq!(*n, 2);
//!   ```
//!   
//! - **`Result::combine`** and **`Option::combine`** to zip pairs of objects:
//!   
//!   ```rust
//!   use stdext::prelude::*;
//!   
//!   let x = Some(1);
//!   let y = Some("hi");
//!   let z = None::<u8>;
//!   
//!   assert_eq!(x.combine(y), Some((1, "hi")));
//!   assert_eq!(x.combine(z), None);
//!
//!   let x = Ok(1);
//!   let y = Ok("hi");
//!   let z: Result<i32, &str> = Err("error");
//!   let z2: Result<i32, &str> = Err("other_error");
//!
//!   assert_eq!(x.combine(y), Ok((1, "hi")));
//!   assert_eq!(x.combine(z), Err("error"));
//!   assert_eq!(z.combine(z2), Err("error"));
//!   ```
//!
//! - New handy macros (mostly for development purposes):
//!   
//!   ```rust
//!   use stdext::{compile_warning, function_name};
//!
//!   fn sample_function() {
//!     println!("This function is called {}", function_name!());
//!
//!     compile_warning!("This function must do something else...");
//!   }
//!   ```

#![warn(missing_docs, unreachable_pub)]

pub mod duration;
#[macro_use]
pub mod macros;
pub mod num;
pub mod option;
pub mod result;
pub mod str;
pub mod sync;
pub mod vec;

/// A "prelude" module which re-exports all the extension traits for the simple library usage.
pub mod prelude {
    pub use crate::{
        duration::*,
        num::{float_convert::*, integer::*},
        option::*,
        result::*,
        str::*,
        sync::{mutex::*, rw_lock::*},
        vec::*,
    };
}
