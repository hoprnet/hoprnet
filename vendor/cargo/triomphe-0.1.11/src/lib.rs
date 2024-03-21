// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fork of Arc. This has the following advantages over std::sync::Arc:
//!
//! * `triomphe::Arc` doesn't support weak references: we save space by excluding the weak reference count, and we don't do extra read-modify-update operations to handle the possibility of weak references.
//! * `triomphe::UniqueArc` allows one to construct a temporarily-mutable `Arc` which can be converted to a regular `triomphe::Arc` later
//! * `triomphe::OffsetArc` can be used transparently from C++ code and is compatible with (and can be converted to/from) `triomphe::Arc`
//! * `triomphe::ArcBorrow` is functionally similar to `&triomphe::Arc<T>`, however in memory it's simply `&T`. This makes it more flexible for FFI; the source of the borrow need not be an `Arc` pinned on the stack (and can instead be a pointer from C++, or an `OffsetArc`). Additionally, this helps avoid pointer-chasing.
//! * `triomphe::Arc` has can be constructed for dynamically-sized types via `from_header_and_iter`
//! * `triomphe::ThinArc` provides thin-pointer `Arc`s to dynamically sized types
//! * `triomphe::ArcUnion` is union of two `triomphe:Arc`s which fits inside one word of memory

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "arc-swap")]
extern crate arc_swap;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "stable_deref_trait")]
extern crate stable_deref_trait;
#[cfg(feature = "unsize")]
extern crate unsize;

/// Calculates the offset of the specified field from the start of the named struct.
/// This macro is impossible to be const until feature(const_ptr_offset_from) is stable.
macro_rules! offset_of {
    ($ty: path, $field: tt) => {{
        // ensure the type is a named struct
        // ensure the field exists and is accessible
        let $ty { $field: _, .. };

        let uninit = <::core::mem::MaybeUninit<$ty>>::uninit(); // const since 1.36

        let base_ptr: *const $ty = uninit.as_ptr(); // const since 1.59

        #[allow(unused_unsafe)]
        let field_ptr = unsafe { ::core::ptr::addr_of!((*base_ptr).$field) }; // since 1.51

        // // the const version requires feature(const_ptr_offset_from)
        // // https://github.com/rust-lang/rust/issues/92980
        // #[allow(unused_unsafe)]
        // unsafe { (field_ptr as *const u8).offset_from(base_ptr as *const u8) as usize }

        (field_ptr as usize) - (base_ptr as usize)
    }};
}

mod arc;
mod arc_borrow;
#[cfg(feature = "arc-swap")]
mod arc_swap_support;
mod arc_union;
mod header;
mod iterator_as_exact_size_iterator;
mod offset_arc;
mod thin_arc;
mod unique_arc;

pub use arc::*;
pub use arc_borrow::*;
pub use arc_union::*;
pub use header::*;
pub use offset_arc::*;
pub use thin_arc::*;
pub use unique_arc::*;

#[cfg(feature = "std")]
use std::process::abort;

// `no_std`-compatible abort by forcing a panic while already panicking.
#[cfg(not(feature = "std"))]
#[cold]
fn abort() -> ! {
    struct PanicOnDrop;
    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!()
        }
    }
    let _double_panicer = PanicOnDrop;
    panic!();
}
