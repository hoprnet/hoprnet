//-
// Copyright 2017, 2018 The proptest developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Arbitrary implementations for `std::panic`.

use std::panic::AssertUnwindSafe;

wrap_ctor!(AssertUnwindSafe, AssertUnwindSafe);

#[cfg(test)]
mod test {
    no_panic_test!(assert_unwind_safe => AssertUnwindSafe<u8>);
}
