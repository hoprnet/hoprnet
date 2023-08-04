# `std` extensions

**Status:**
[![CI](https://github.com/popzxc/stdext-rs/workflows/CI/badge.svg)](https://github.com/popzxc/stdext-rs/actions)

**Project info:**
[![Docs.rs](https://docs.rs/stdext/badge.svg)](https://docs.rs/stdext)
[![Latest Version](https://img.shields.io/crates/v/stdext.svg)](https://crates.io/crates/stdext)
[![License](https://img.shields.io/github/license/popzxc/stdext-rs.svg)](https://github.com/popzxc/stdext-rs)
![Rust 1.53+ required](https://img.shields.io/badge/rust-1.53+-blue.svg?label=Rust)

Additional features for the Rust standard library.

## Description

This crate contains enhancements to the Rust standard library structures, useful for
broad audience, but not yet implemented (or stabilized) in `std`.

Crate is designed to be lightweight (no external dependencies!) and provide essential
functionality which possible can get to the `std` some day.

The minimal supported Rust version for release 0.3 is 1.53. However, if you need to use
this crate with an older version of the compiler, check out release 0.2; there is a good
chance that it will suit your needs.

## Highlights

- `Integer` super-trait that is implemented for all the built-in integers
  and reflects the common part of their interfaces.
  
  ```rust
  use stdext::prelude::*;
  
  fn accepts_any_integer<I: Integer>(a: I, b: I) {
    println!("{}", (a + b).count_ones());
  }
  ```
  
- Safe conversions from floating numbers to integers.

  ```rust
  use stdext::prelude::FloatConvert;
  
  let valid: Option<u8> = 10.5f32.checked_floor();
  let too_big: Option<u8> = 256f32.checked_floor();
  let nan: Option<u8> = f32::NAN.checked_floor();
  
  assert_eq!(valid, Some(10u8));
  assert_eq!(too_big, None);
  assert_eq!(nan, None);
  ```

- Convenient builder methods for **`Duration`**:
  
  ```rust
  use std::time::Duration;
  use stdext::prelude::*;

  let duration = Duration::from_minutes(1).add_secs(5).add_micros(100);
  assert_eq!(duration, Duration::new(65, 100_000));
  ```

- Panicking version for **`RwLock::read`**, **`RwLock::write`** and **`Mutex::lock`** (for
  fellows who don't really handle the lock poisoning):

  ```rust
  use std::sync::{Arc, RwLock};
  use stdext::prelude::*;
  
  let lock = Arc::new(RwLock::new(1));
  {
      let mut n = lock.force_write(); // Instead of `.write().unwrap()`.
      *n = 2;
  }
  let n = lock.force_read();
  assert_eq!(*n, 2);
  ```
  
- **`Result::combine`** and **`Option::combine`** to zip pairs of objects:
  
  ```rust
  use stdext::prelude::*;
  
  let x = Some(1);
  let y = Some("hi");
  let z = None::<u8>;
  
  assert_eq!(x.combine(y), Some((1, "hi")));
  assert_eq!(x.combine(z), None);

  let x = Ok(1);
  let y = Ok("hi");
  let z: Result<i32, &str> = Err("error");
  let z2: Result<i32, &str> = Err("other_error");

  assert_eq!(x.combine(y), Ok((1, "hi")));
  assert_eq!(x.combine(z), Err("error"));
  assert_eq!(z.combine(z2), Err("error"));
  ```

- New handy macros (mostly for development purposes):
  
  ```rust
  use stdext::{compile_warning, function_name};

  fn sample_function() {
    println!("This function is called {}", function_name!());

    compile_warning!("This function must do something else...");
  }
  ```

...and other features. For a full list, check out the [crate documentation](https://docs.rs/stdext/).

## Motivation

Standard library is great, and it becomes even better through time. However, a time gap between proposing
a new feature and getting it stabilized is way too big.

This crate, however, can be updated quickly and the feature will be usable on the stable Rust soon after
implementation.

## Contributing

If you want to contribute to this project, please read the [contributing guide](CONTRIBUTING.md).

## LICENSE

`stdext` library is licensed under the MIT License. See [LICENSE](LICENSE) for details.
