// Copyright 2019-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Exports from `std`, `core` and `alloc` crates.
//!
//! Guarantees a stable interface between `std` and `no_std` modes.

#[cfg(not(feature = "std"))]
extern crate alloc;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::{
            any,
            borrow,
            boxed,
            cmp,
            collections,
            fmt,
            format,
            hash,
            marker,
            mem,
            num,
            ops,
            string,
            sync,
            time,
            vec,
            rc,
            iter,
        };
    } else {
        pub use alloc::{
            borrow,
            boxed,
            collections,
            format,
            string,
            sync,
            vec,
            rc
        };

        pub use core::{
            any,
            cmp,
            fmt,
            hash,
            marker,
            mem,
            num,
            ops,
            time,
            iter,
        };
    }
}
