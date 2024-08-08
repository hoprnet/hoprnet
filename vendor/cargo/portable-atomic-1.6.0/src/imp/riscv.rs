// SPDX-License-Identifier: Apache-2.0 OR MIT

// Atomic load/store implementation on RISC-V.
//
// This is for RISC-V targets without atomic CAS. (rustc doesn't provide atomics
// at all on such targets. https://github.com/rust-lang/rust/pull/114499)
//
// Also, optionally provides RMW implementation when force-amo is enabled.
//
// Refs:
// - RISC-V Instruction Set Manual Volume I: Unprivileged ISA
//   https://riscv.org/wp-content/uploads/2019/12/riscv-spec-20191213.pdf
// - RISC-V Atomics ABI Specification
//   https://github.com/riscv-non-isa/riscv-elf-psabi-doc/blob/HEAD/riscv-atomic.adoc
// - "Mappings from C/C++ primitives to RISC-V primitives." table in RISC-V Instruction Set Manual:
//   https://five-embeddev.com/riscv-isa-manual/latest/memory.html#sec:memory:porting
// - atomic-maybe-uninit https://github.com/taiki-e/atomic-maybe-uninit
//
// Generated asm:
// - riscv64gc https://godbolt.org/z/EETebx7TE
// - riscv32imac https://godbolt.org/z/8zzv73bKh

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;
use core::{cell::UnsafeCell, sync::atomic::Ordering};

#[cfg(any(test, portable_atomic_force_amo))]
macro_rules! atomic_rmw_amo_order {
    ($op:ident, $order:ident) => {
        match $order {
            Ordering::Relaxed => $op!(""),
            Ordering::Acquire => $op!(".aq"),
            Ordering::Release => $op!(".rl"),
            // AcqRel and SeqCst RMWs are equivalent.
            Ordering::AcqRel | Ordering::SeqCst => $op!(".aqrl"),
            _ => unreachable!("{:?}", $order),
        }
    };
}
#[cfg(any(test, portable_atomic_force_amo))]
macro_rules! atomic_rmw_amo {
    ($op:ident, $dst:ident, $val:ident, $order:ident, $asm_suffix:tt) => {{
        let out;
        macro_rules! op {
            ($asm_order:tt) => {
                // SAFETY: The user guaranteed that the AMO instruction is available in this
                // system by setting the portable_atomic_force_amo and
                // portable_atomic_unsafe_assume_single_core.
                // The caller of this macro must guarantee the validity of the pointer.
                asm!(
                    ".option push",
                    // https://github.com/riscv-non-isa/riscv-asm-manual/blob/HEAD/riscv-asm.md#arch
                    ".option arch, +a",
                    concat!("amo", stringify!($op), ".", $asm_suffix, $asm_order, " {out}, {val}, 0({dst})"),
                    ".option pop",
                    dst = in(reg) ptr_reg!($dst),
                    val = in(reg) $val,
                    out = lateout(reg) out,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw_amo_order!(op, $order);
        out
    }};
}
// 32-bit val.wrapping_shl(shift) but no extra `& (u32::BITS - 1)`
#[cfg(any(test, portable_atomic_force_amo))]
#[inline]
fn sllw(val: u32, shift: u32) -> u32 {
    // SAFETY: Calling sll{,w} is safe.
    unsafe {
        let out;
        #[cfg(target_arch = "riscv32")]
        asm!("sll {out}, {val}, {shift}", out = lateout(reg) out, val = in(reg) val, shift = in(reg) shift, options(pure, nomem, nostack, preserves_flags));
        #[cfg(target_arch = "riscv64")]
        asm!("sllw {out}, {val}, {shift}", out = lateout(reg) out, val = in(reg) val, shift = in(reg) shift, options(pure, nomem, nostack, preserves_flags));
        out
    }
}
// 32-bit val.wrapping_shr(shift) but no extra `& (u32::BITS - 1)`
#[cfg(any(test, portable_atomic_force_amo))]
#[inline]
fn srlw(val: u32, shift: u32) -> u32 {
    // SAFETY: Calling srl{,w} is safe.
    unsafe {
        let out;
        #[cfg(target_arch = "riscv32")]
        asm!("srl {out}, {val}, {shift}", out = lateout(reg) out, val = in(reg) val, shift = in(reg) shift, options(pure, nomem, nostack, preserves_flags));
        #[cfg(target_arch = "riscv64")]
        asm!("srlw {out}, {val}, {shift}", out = lateout(reg) out, val = in(reg) val, shift = in(reg) shift, options(pure, nomem, nostack, preserves_flags));
        out
    }
}

macro_rules! atomic_load_store {
    ($([$($generics:tt)*])? $atomic_type:ident, $value_type:ty, $asm_suffix:tt) => {
        #[repr(transparent)]
        pub(crate) struct $atomic_type $(<$($generics)*>)? {
            v: UnsafeCell<$value_type>,
        }

        // Send is implicitly implemented for atomic integers, but not for atomic pointers.
        // SAFETY: any data races are prevented by atomic operations.
        unsafe impl $(<$($generics)*>)? Send for $atomic_type $(<$($generics)*>)? {}
        // SAFETY: any data races are prevented by atomic operations.
        unsafe impl $(<$($generics)*>)? Sync for $atomic_type $(<$($generics)*>)? {}

        #[cfg(any(test, not(portable_atomic_unsafe_assume_single_core)))]
        impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
            #[inline]
            pub(crate) const fn new(v: $value_type) -> Self {
                Self { v: UnsafeCell::new(v) }
            }

            #[inline]
            pub(crate) fn is_lock_free() -> bool {
                Self::is_always_lock_free()
            }
            #[inline]
            pub(crate) const fn is_always_lock_free() -> bool {
                true
            }

            #[inline]
            pub(crate) fn get_mut(&mut self) -> &mut $value_type {
                // SAFETY: the mutable reference guarantees unique ownership.
                // (UnsafeCell::get_mut requires Rust 1.50)
                unsafe { &mut *self.v.get() }
            }

            #[inline]
            pub(crate) fn into_inner(self) -> $value_type {
                 self.v.into_inner()
            }

            #[inline]
            pub(crate) const fn as_ptr(&self) -> *mut $value_type {
                self.v.get()
            }
        }
        impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn load(&self, order: Ordering) -> $value_type {
                crate::utils::assert_load_ordering(order);
                let src = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe {
                    let out;
                    match order {
                        Ordering::Relaxed => {
                            asm!(
                                concat!("l", $asm_suffix, " {out}, 0({src})"),
                                src = in(reg) ptr_reg!(src),
                                out = lateout(reg) out,
                                options(nostack, preserves_flags, readonly),
                            );
                        }
                        Ordering::Acquire => {
                            asm!(
                                concat!("l", $asm_suffix, " {out}, 0({src})"),
                                "fence r, rw",
                                src = in(reg) ptr_reg!(src),
                                out = lateout(reg) out,
                                options(nostack, preserves_flags),
                            );
                        }
                        Ordering::SeqCst => {
                            asm!(
                                "fence rw, rw",
                                concat!("l", $asm_suffix, " {out}, 0({src})"),
                                "fence r, rw",
                                src = in(reg) ptr_reg!(src),
                                out = lateout(reg) out,
                                options(nostack, preserves_flags),
                            );
                        }
                        _ => unreachable!("{:?}", order),
                    }
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
                    match order {
                        Ordering::Relaxed => {
                            asm!(
                                concat!("s", $asm_suffix, " {val}, 0({dst})"),
                                dst = in(reg) ptr_reg!(dst),
                                val = in(reg) val,
                                options(nostack, preserves_flags),
                            );
                        }
                        // Release and SeqCst stores are equivalent.
                        Ordering::Release | Ordering::SeqCst => {
                            asm!(
                                "fence rw, w",
                                concat!("s", $asm_suffix, " {val}, 0({dst})"),
                                dst = in(reg) ptr_reg!(dst),
                                val = in(reg) val,
                                options(nostack, preserves_flags),
                            );
                        }
                        _ => unreachable!("{:?}", order),
                    }
                }
            }
        }
    };
}

macro_rules! atomic_ptr {
    ($([$($generics:tt)*])? $atomic_type:ident, $value_type:ty, $asm_suffix:tt) => {
        atomic_load_store!($([$($generics)*])? $atomic_type, $value_type, $asm_suffix);
        #[cfg(portable_atomic_force_amo)]
        impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
            #[inline]
            pub(crate) fn swap(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(swap, dst, val, order, $asm_suffix) }
            }
        }
    };
}

macro_rules! atomic {
    ($atomic_type:ident, $value_type:ty, $asm_suffix:tt, $max:tt, $min:tt) => {
        atomic_load_store!($atomic_type, $value_type, $asm_suffix);
        // There is no amo{sub,nand,neg}.
        #[cfg(any(test, portable_atomic_force_amo))]
        impl $atomic_type {
            #[inline]
            pub(crate) fn swap(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(swap, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_add(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(add, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_sub(&self, val: $value_type, order: Ordering) -> $value_type {
                self.fetch_add(val.wrapping_neg(), order)
            }

            #[inline]
            pub(crate) fn fetch_and(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(and, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_or(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(or, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_xor(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!(xor, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_not(&self, order: Ordering) -> $value_type {
                self.fetch_xor(!0, order)
            }

            #[inline]
            pub(crate) fn fetch_max(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!($max, dst, val, order, $asm_suffix) }
            }

            #[inline]
            pub(crate) fn fetch_min(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                unsafe { atomic_rmw_amo!($min, dst, val, order, $asm_suffix) }
            }
        }
    };
}

macro_rules! atomic_sub_word {
    ($atomic_type:ident, $value_type:ty, $unsigned_type:ty, $asm_suffix:tt) => {
        atomic_load_store!($atomic_type, $value_type, $asm_suffix);
        #[cfg(any(test, portable_atomic_force_amo))]
        impl $atomic_type {
            #[inline]
            pub(crate) fn fetch_and(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                let (dst, shift, mask) = crate::utils::create_sub_word_mask_values(dst);
                let mask = !sllw(mask as u32, shift as u32);
                // TODO: use zero_extend helper instead of cast for val.
                let val = sllw(val as $unsigned_type as u32, shift as u32);
                let val = val | mask;
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                let out: u32 = unsafe { atomic_rmw_amo!(and, dst, val, order, "w") };
                srlw(out, shift as u32) as $unsigned_type as $value_type
            }

            #[inline]
            pub(crate) fn fetch_or(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                let (dst, shift, _mask) = crate::utils::create_sub_word_mask_values(dst);
                // TODO: use zero_extend helper instead of cast for val.
                let val = sllw(val as $unsigned_type as u32, shift as u32);
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                let out: u32 = unsafe { atomic_rmw_amo!(or, dst, val, order, "w") };
                srlw(out, shift as u32) as $unsigned_type as $value_type
            }

            #[inline]
            pub(crate) fn fetch_xor(&self, val: $value_type, order: Ordering) -> $value_type {
                let dst = self.v.get();
                let (dst, shift, _mask) = crate::utils::create_sub_word_mask_values(dst);
                // TODO: use zero_extend helper instead of cast for val.
                let val = sllw(val as $unsigned_type as u32, shift as u32);
                // SAFETY: any data races are prevented by atomic intrinsics and the raw
                // pointer passed in is valid because we got it from a reference.
                let out: u32 = unsafe { atomic_rmw_amo!(xor, dst, val, order, "w") };
                srlw(out, shift as u32) as $unsigned_type as $value_type
            }

            #[inline]
            pub(crate) fn fetch_not(&self, order: Ordering) -> $value_type {
                self.fetch_xor(!0, order)
            }
        }
    };
}

atomic_sub_word!(AtomicI8, i8, u8, "b");
atomic_sub_word!(AtomicU8, u8, u8, "b");
atomic_sub_word!(AtomicI16, i16, u16, "h");
atomic_sub_word!(AtomicU16, u16, u16, "h");
atomic!(AtomicI32, i32, "w", max, min);
atomic!(AtomicU32, u32, "w", maxu, minu);
#[cfg(target_arch = "riscv64")]
atomic!(AtomicI64, i64, "d", max, min);
#[cfg(target_arch = "riscv64")]
atomic!(AtomicU64, u64, "d", maxu, minu);
#[cfg(target_pointer_width = "32")]
atomic!(AtomicIsize, isize, "w", max, min);
#[cfg(target_pointer_width = "32")]
atomic!(AtomicUsize, usize, "w", maxu, minu);
#[cfg(target_pointer_width = "32")]
atomic_ptr!([T] AtomicPtr, *mut T, "w");
#[cfg(target_pointer_width = "64")]
atomic!(AtomicIsize, isize, "d", max, min);
#[cfg(target_pointer_width = "64")]
atomic!(AtomicUsize, usize, "d", maxu, minu);
#[cfg(target_pointer_width = "64")]
atomic_ptr!([T] AtomicPtr, *mut T, "d");

#[cfg(test)]
mod tests {
    use super::*;

    test_atomic_ptr_load_store!();
    test_atomic_int_load_store!(i8);
    test_atomic_int_load_store!(u8);
    test_atomic_int_load_store!(i16);
    test_atomic_int_load_store!(u16);
    test_atomic_int_load_store!(i32);
    test_atomic_int_load_store!(u32);
    #[cfg(target_arch = "riscv64")]
    test_atomic_int_load_store!(i64);
    #[cfg(target_arch = "riscv64")]
    test_atomic_int_load_store!(u64);
    test_atomic_int_load_store!(isize);
    test_atomic_int_load_store!(usize);

    macro_rules! test_atomic_int_amo {
        ($int_type:ident) => {
            paste::paste! {
                #[allow(
                    clippy::alloc_instead_of_core,
                    clippy::std_instead_of_alloc,
                    clippy::std_instead_of_core,
                    clippy::undocumented_unsafe_blocks
                )]
                mod [<test_atomic_ $int_type _amo>] {
                    use super::*;
                    test_atomic_int_amo!([<Atomic $int_type:camel>], $int_type);
                }
            }
        };
        ($atomic_type:ty, $int_type:ident) => {
            ::quickcheck::quickcheck! {
                fn quickcheck_swap(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.swap(y, order), x);
                        assert_eq!(a.swap(x, order), y);
                    }
                    true
                }
                fn quickcheck_fetch_add(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_add(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), x.wrapping_add(y));
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_add(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), y.wrapping_add(x));
                    }
                    true
                }
                fn quickcheck_fetch_sub(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_sub(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), x.wrapping_sub(y));
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_sub(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), y.wrapping_sub(x));
                    }
                    true
                }
                fn quickcheck_fetch_and(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_and(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), x & y);
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_and(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), y & x);
                    }
                    true
                }
                fn quickcheck_fetch_or(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_or(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), x | y);
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_or(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), y | x);
                    }
                    true
                }
                fn quickcheck_fetch_xor(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_xor(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), x ^ y);
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_xor(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), y ^ x);
                    }
                    true
                }
                fn quickcheck_fetch_max(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_max(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), core::cmp::max(x, y));
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_max(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), core::cmp::max(y, x));
                    }
                    true
                }
                fn quickcheck_fetch_min(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_min(y, order), x);
                        assert_eq!(a.load(Ordering::Relaxed), core::cmp::min(x, y));
                        let a = <$atomic_type>::new(y);
                        assert_eq!(a.fetch_min(x, order), y);
                        assert_eq!(a.load(Ordering::Relaxed), core::cmp::min(y, x));
                    }
                    true
                }
                fn quickcheck_fetch_not(x: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        let a = <$atomic_type>::new(x);
                        assert_eq!(a.fetch_not(order), x);
                        assert_eq!(a.load(Ordering::Relaxed), !x);
                        assert_eq!(a.fetch_not(order), !x);
                        assert_eq!(a.load(Ordering::Relaxed), x);
                    }
                    true
                }
            }
        };
    }
    macro_rules! test_atomic_int_amo_sub_word {
        ($int_type:ident) => {
            paste::paste! {
                #[allow(
                    clippy::alloc_instead_of_core,
                    clippy::std_instead_of_alloc,
                    clippy::std_instead_of_core,
                    clippy::undocumented_unsafe_blocks
                )]
                mod [<test_atomic_ $int_type _amo>] {
                    use super::*;
                    test_atomic_int_amo_sub_word!([<Atomic $int_type:camel>], $int_type);
                }
            }
        };
        ($atomic_type:ty, $int_type:ident) => {
            use crate::tests::helper::*;
            ::quickcheck::quickcheck! {
                fn quickcheck_fetch_and(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        for base in [0, !0] {
                            let mut arr = Align16([
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                            ]);
                            let a_idx = fastrand::usize(3..=6);
                            arr.0[a_idx] = <$atomic_type>::new(x);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_and(y, order), x);
                            assert_eq!(a.load(Ordering::Relaxed), x & y);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            arr.0[a_idx] = <$atomic_type>::new(y);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_and(x, order), y);
                            assert_eq!(a.load(Ordering::Relaxed), y & x);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                        }
                    }
                    true
                }
                fn quickcheck_fetch_or(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        for base in [0, !0] {
                            let mut arr = Align16([
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                            ]);
                            let a_idx = fastrand::usize(3..=6);
                            arr.0[a_idx] = <$atomic_type>::new(x);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_or(y, order), x);
                            assert_eq!(a.load(Ordering::Relaxed), x | y);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            arr.0[a_idx] = <$atomic_type>::new(y);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_or(x, order), y);
                            assert_eq!(a.load(Ordering::Relaxed), y | x);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                        }
                    }
                    true
                }
                fn quickcheck_fetch_xor(x: $int_type, y: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        for base in [0, !0] {
                            let mut arr = Align16([
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                            ]);
                            let a_idx = fastrand::usize(3..=6);
                            arr.0[a_idx] = <$atomic_type>::new(x);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_xor(y, order), x);
                            assert_eq!(a.load(Ordering::Relaxed), x ^ y);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            arr.0[a_idx] = <$atomic_type>::new(y);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_xor(x, order), y);
                            assert_eq!(a.load(Ordering::Relaxed), y ^ x);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                        }
                    }
                    true
                }
                fn quickcheck_fetch_not(x: $int_type) -> bool {
                    for &order in &test_helper::SWAP_ORDERINGS {
                        for base in [0, !0] {
                            let mut arr = Align16([
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                                <$atomic_type>::new(base),
                            ]);
                            let a_idx = fastrand::usize(3..=6);
                            arr.0[a_idx] = <$atomic_type>::new(x);
                            let a = &arr.0[a_idx];
                            assert_eq!(a.fetch_not(order), x);
                            assert_eq!(a.load(Ordering::Relaxed), !x);
                            assert_eq!(a.fetch_not(order), !x);
                            assert_eq!(a.load(Ordering::Relaxed), x);
                            for i in 0..a_idx {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                            for i in a_idx + 1..arr.0.len() {
                                assert_eq!(arr.0[i].load(Ordering::Relaxed), base, "invalid value written");
                            }
                        }
                    }
                    true
                }
            }
        };
    }
    test_atomic_int_amo_sub_word!(i8);
    test_atomic_int_amo_sub_word!(u8);
    test_atomic_int_amo_sub_word!(i16);
    test_atomic_int_amo_sub_word!(u16);
    test_atomic_int_amo!(i32);
    test_atomic_int_amo!(u32);
    #[cfg(target_arch = "riscv64")]
    test_atomic_int_amo!(i64);
    #[cfg(target_arch = "riscv64")]
    test_atomic_int_amo!(u64);
    test_atomic_int_amo!(isize);
    test_atomic_int_amo!(usize);
}
