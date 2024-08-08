// SPDX-License-Identifier: Apache-2.0 OR MIT

// Atomic load/store implementation on MSP430.
//
// Adapted from https://github.com/pftbest/msp430-atomic.
// Including https://github.com/pftbest/msp430-atomic/pull/4 for a compile error fix.
// Including https://github.com/pftbest/msp430-atomic/pull/5 for a soundness bug fix.
//
// Operations not supported here are provided by disabling interrupts.
// See also src/imp/interrupt/msp430.rs.
//
// Note: Ordering is always SeqCst.
//
// Refs: https://www.ti.com/lit/ug/slau208q/slau208q.pdf

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;
#[cfg(any(test, not(feature = "critical-section")))]
use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;

/// An atomic fence.
///
/// # Panics
///
/// Panics if `order` is [`Relaxed`](Ordering::Relaxed).
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub fn fence(order: Ordering) {
    match order {
        Ordering::Relaxed => panic!("there is no such thing as a relaxed fence"),
        // MSP430 is single-core and a compiler fence works as an atomic fence.
        _ => compiler_fence(order),
    }
}

/// A compiler memory fence.
///
/// # Panics
///
/// Panics if `order` is [`Relaxed`](Ordering::Relaxed).
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub fn compiler_fence(order: Ordering) {
    match order {
        Ordering::Relaxed => panic!("there is no such thing as a relaxed compiler fence"),
        _ => {}
    }
    // SAFETY: using an empty asm is safe.
    unsafe {
        // Do not use `nomem` and `readonly` because prevent preceding and subsequent memory accesses from being reordered.
        #[cfg(not(portable_atomic_no_asm))]
        asm!("", options(nostack, preserves_flags));
        #[cfg(portable_atomic_no_asm)]
        llvm_asm!("" ::: "memory" : "volatile");
    }
}

macro_rules! atomic {
    (load_store, $([$($generics:tt)*])? $atomic_type:ident, $value_type:ty, $asm_suffix:tt) => {
        #[cfg(any(test, not(feature = "critical-section")))]
        #[repr(transparent)]
        pub(crate) struct $atomic_type $(<$($generics)*>)? {
            v: UnsafeCell<$value_type>,
        }

        #[cfg(any(test, not(feature = "critical-section")))]
        // Send is implicitly implemented for atomic integers, but not for atomic pointers.
        // SAFETY: any data races are prevented by atomic operations.
        unsafe impl $(<$($generics)*>)? Send for $atomic_type $(<$($generics)*>)? {}
        #[cfg(any(test, not(feature = "critical-section")))]
        // SAFETY: any data races are prevented by atomic operations.
        unsafe impl $(<$($generics)*>)? Sync for $atomic_type $(<$($generics)*>)? {}

        #[cfg(any(test, not(feature = "critical-section")))]
        impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
            #[cfg(test)]
            #[inline]
            pub(crate) const fn new(v: $value_type) -> Self {
                Self { v: UnsafeCell::new(v) }
            }

            #[cfg(test)]
            #[inline]
            pub(crate) fn is_lock_free() -> bool {
                Self::is_always_lock_free()
            }
            #[cfg(test)]
            #[inline]
            pub(crate) const fn is_always_lock_free() -> bool {
                true
            }

            #[cfg(test)]
            #[inline]
            pub(crate) fn get_mut(&mut self) -> &mut $value_type {
                // SAFETY: the mutable reference guarantees unique ownership.
                // (UnsafeCell::get_mut requires Rust 1.50)
                unsafe { &mut *self.v.get() }
            }

            #[cfg(test)]
            #[inline]
            pub(crate) fn into_inner(self) -> $value_type {
                 self.v.into_inner()
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn load(&self, order: Ordering) -> $value_type {
                crate::utils::assert_load_ordering(order);
                let src = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    let out;
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("mov", $asm_suffix, " @{src}, {out}"),
                        src = in(reg) src,
                        out = lateout(reg) out,
                        options(nostack, preserves_flags),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("mov", $asm_suffix, " $1, $0")
                        : "=r"(out) : "*m"(src) : "memory" : "volatile"
                    );
                    out
                }
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn store(&self, val: $value_type, order: Ordering) {
                crate::utils::assert_store_ordering(order);
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("mov", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        options(nostack, preserves_flags),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("mov", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }
        }
    };
    ($([$($generics:tt)*])? $atomic_type:ident, $value_type:ty, $asm_suffix:tt) => {
        atomic!(load_store, $([$($generics)*])? $atomic_type, $value_type, $asm_suffix);
        #[cfg(any(test, not(feature = "critical-section")))]
        impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
            #[inline]
            pub(crate) fn add(&self, val: $value_type, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("add", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        // Do not use `preserves_flags` because ADD modifies the V, N, Z, and C bits of the status register.
                        options(nostack),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("add", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }

            #[inline]
            pub(crate) fn sub(&self, val: $value_type, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("sub", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        // Do not use `preserves_flags` because SUB modifies the V, N, Z, and C bits of the status register.
                        options(nostack),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("sub", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }

            #[inline]
            pub(crate) fn and(&self, val: $value_type, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("and", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        // Do not use `preserves_flags` because AND modifies the V, N, Z, and C bits of the status register.
                        options(nostack),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("and", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }

            #[inline]
            pub(crate) fn or(&self, val: $value_type, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("bis", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        options(nostack, preserves_flags),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("bis", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }

            #[inline]
            pub(crate) fn xor(&self, val: $value_type, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("xor", $asm_suffix, " {val}, 0({dst})"),
                        dst = in(reg) dst,
                        val = in(reg) val,
                        // Do not use `preserves_flags` because XOR modifies the V, N, Z, and C bits of the status register.
                        options(nostack),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("xor", $asm_suffix, " $1, $0")
                        :: "*m"(dst), "ir"(val) : "memory" : "volatile"
                    );
                }
            }

            #[inline]
            pub(crate) fn not(&self, _order: Ordering) {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    #[cfg(not(portable_atomic_no_asm))]
                    asm!(
                        concat!("inv", $asm_suffix, " 0({dst})"),
                        dst = in(reg) dst,
                        // Do not use `preserves_flags` because INV modifies the V, N, Z, and C bits of the status register.
                        options(nostack),
                    );
                    #[cfg(portable_atomic_no_asm)]
                    llvm_asm!(
                        concat!("inv", $asm_suffix, " $0")
                        :: "*m"(dst) : "memory" : "volatile"
                    );
                }
            }
        }
    }
}

atomic!(AtomicI8, i8, ".b");
atomic!(AtomicU8, u8, ".b");
atomic!(AtomicI16, i16, ".w");
atomic!(AtomicU16, u16, ".w");
atomic!(AtomicIsize, isize, ".w");
atomic!(AtomicUsize, usize, ".w");
atomic!(load_store, [T] AtomicPtr, *mut T, ".w");
