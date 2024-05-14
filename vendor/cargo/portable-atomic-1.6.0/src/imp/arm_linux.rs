// SPDX-License-Identifier: Apache-2.0 OR MIT

// 64-bit atomic implementation using kuser_cmpxchg64 on pre-v6 ARM Linux/Android.
//
// Refs:
// - https://www.kernel.org/doc/Documentation/arm/kernel_user_helpers.txt
// - https://github.com/rust-lang/compiler-builtins/blob/0.1.88/src/arm_linux.rs
//
// Note: On Miri and ThreadSanitizer which do not support inline assembly, we don't use
// this module and use fallback implementation instead.

// TODO: Since Rust 1.64, the Linux kernel requirement for Rust when using std is 3.2+, so it should
// be possible to omit the dynamic kernel version check if the std feature is enabled on Rust 1.64+.
// https://blog.rust-lang.org/2022/08/01/Increasing-glibc-kernel-requirements.html

#[path = "fallback/outline_atomics.rs"]
mod fallback;

use core::{arch::asm, cell::UnsafeCell, mem, sync::atomic::Ordering};

use crate::utils::{Pair, U64};

// https://www.kernel.org/doc/Documentation/arm/kernel_user_helpers.txt
const KUSER_HELPER_VERSION: usize = 0xFFFF0FFC;
// __kuser_helper_version >= 5 (kernel version 3.1+)
const KUSER_CMPXCHG64: usize = 0xFFFF0F60;
#[inline]
fn __kuser_helper_version() -> i32 {
    use core::sync::atomic::AtomicI32;

    static CACHE: AtomicI32 = AtomicI32::new(0);
    let mut v = CACHE.load(Ordering::Relaxed);
    if v != 0 {
        return v;
    }
    // SAFETY: core assumes that at least __kuser_memory_barrier (__kuser_helper_version >= 3) is
    // available on this platform. __kuser_helper_version is always available on such a platform.
    v = unsafe { (KUSER_HELPER_VERSION as *const i32).read() };
    CACHE.store(v, Ordering::Relaxed);
    v
}
#[inline]
fn has_kuser_cmpxchg64() -> bool {
    // Note: detect_false cfg is intended to make it easy for portable-atomic developers to
    // test cases such as has_cmpxchg16b == false, has_lse == false,
    // __kuser_helper_version < 5, etc., and is not a public API.
    if cfg!(portable_atomic_test_outline_atomics_detect_false) {
        return false;
    }
    __kuser_helper_version() >= 5
}
#[inline]
unsafe fn __kuser_cmpxchg64(old_val: *const u64, new_val: *const u64, ptr: *mut u64) -> bool {
    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let f: extern "C" fn(*const u64, *const u64, *mut u64) -> u32 =
            mem::transmute(KUSER_CMPXCHG64 as *const ());
        f(old_val, new_val, ptr) == 0
    }
}

// 64-bit atomic load by two 32-bit atomic loads.
#[inline]
unsafe fn byte_wise_atomic_load(src: *const u64) -> u64 {
    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let (out_lo, out_hi);
        asm!(
            "ldr {out_lo}, [{src}]",
            "ldr {out_hi}, [{src}, #4]",
            src = in(reg) src,
            out_lo = out(reg) out_lo,
            out_hi = out(reg) out_hi,
            options(pure, nostack, preserves_flags, readonly),
        );
        U64 { pair: Pair { lo: out_lo, hi: out_hi } }.whole
    }
}

#[inline(always)]
unsafe fn atomic_update_kuser_cmpxchg64<F>(dst: *mut u64, mut f: F) -> u64
where
    F: FnMut(u64) -> u64,
{
    debug_assert!(dst as usize % 8 == 0);
    debug_assert!(has_kuser_cmpxchg64());
    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        loop {
            // This is not single-copy atomic reads, but this is ok because subsequent
            // CAS will check for consistency.
            //
            // Note that the C++20 memory model does not allow mixed-sized atomic access,
            // so we must use inline assembly to implement byte_wise_atomic_load.
            // (i.e., byte-wise atomic based on the standard library's atomic types
            // cannot be used here).
            let prev = byte_wise_atomic_load(dst);
            let next = f(prev);
            if __kuser_cmpxchg64(&prev, &next, dst) {
                return prev;
            }
        }
    }
}

macro_rules! atomic_with_ifunc {
    (
        unsafe fn $name:ident($($arg:tt)*) $(-> $ret_ty:ty)? { $($kuser_cmpxchg64_fn_body:tt)* }
        fallback = $seqcst_fallback_fn:ident
    ) => {
        #[inline]
        unsafe fn $name($($arg)*) $(-> $ret_ty)? {
            unsafe fn kuser_cmpxchg64_fn($($arg)*) $(-> $ret_ty)? {
                $($kuser_cmpxchg64_fn_body)*
            }
            // SAFETY: the caller must uphold the safety contract.
            // we only calls __kuser_cmpxchg64 if it is available.
            unsafe {
                ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                    if has_kuser_cmpxchg64() {
                        kuser_cmpxchg64_fn
                    } else {
                        // Use SeqCst because __kuser_cmpxchg64 is always SeqCst.
                        // https://github.com/torvalds/linux/blob/v6.1/arch/arm/kernel/entry-armv.S#L918-L925
                        fallback::$seqcst_fallback_fn
                    }
                })
            }
        }
    };
}

atomic_with_ifunc! {
    unsafe fn atomic_load(src: *mut u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(src, |old| old) }
    }
    fallback = atomic_load_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_store(dst: *mut u64, val: u64) {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |_| val); }
    }
    fallback = atomic_store_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_swap(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |_| val) }
    }
    fallback = atomic_swap_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_compare_exchange(dst: *mut u64, old: u64, new: u64) -> (u64, bool) {
        // SAFETY: the caller must uphold the safety contract.
        let prev = unsafe {
            atomic_update_kuser_cmpxchg64(dst, |v| if v == old { new } else { v })
        };
        (prev, prev == old)
    }
    fallback = atomic_compare_exchange_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_add(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| x.wrapping_add(val)) }
    }
    fallback = atomic_add_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_sub(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| x.wrapping_sub(val)) }
    }
    fallback = atomic_sub_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_and(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| x & val) }
    }
    fallback = atomic_and_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_nand(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| !(x & val)) }
    }
    fallback = atomic_nand_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_or(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| x | val) }
    }
    fallback = atomic_or_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_xor(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| x ^ val) }
    }
    fallback = atomic_xor_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_max(dst: *mut u64, val: u64) -> u64 {
        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        // SAFETY: the caller must uphold the safety contract.
        unsafe {
            atomic_update_kuser_cmpxchg64(dst, |x| core::cmp::max(x as i64, val as i64) as u64)
        }
    }
    fallback = atomic_max_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_umax(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| core::cmp::max(x, val)) }
    }
    fallback = atomic_umax_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_min(dst: *mut u64, val: u64) -> u64 {
        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        // SAFETY: the caller must uphold the safety contract.
        unsafe {
            atomic_update_kuser_cmpxchg64(dst, |x| core::cmp::min(x as i64, val as i64) as u64)
        }
    }
    fallback = atomic_min_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_umin(dst: *mut u64, val: u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| core::cmp::min(x, val)) }
    }
    fallback = atomic_umin_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_not(dst: *mut u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, |x| !x) }
    }
    fallback = atomic_not_seqcst
}
atomic_with_ifunc! {
    unsafe fn atomic_neg(dst: *mut u64) -> u64 {
        // SAFETY: the caller must uphold the safety contract.
        unsafe { atomic_update_kuser_cmpxchg64(dst, u64::wrapping_neg) }
    }
    fallback = atomic_neg_seqcst
}

macro_rules! atomic64 {
    ($atomic_type:ident, $int_type:ident, $atomic_max:ident, $atomic_min:ident) => {
        #[repr(C, align(8))]
        pub(crate) struct $atomic_type {
            v: UnsafeCell<$int_type>,
        }

        // Send is implicitly implemented.
        // SAFETY: any data races are prevented by the kernel user helper or the lock.
        unsafe impl Sync for $atomic_type {}

        impl_default_no_fetch_ops!($atomic_type, $int_type);
        impl_default_bit_opts!($atomic_type, $int_type);
        impl $atomic_type {
            #[inline]
            pub(crate) const fn new(v: $int_type) -> Self {
                Self { v: UnsafeCell::new(v) }
            }

            #[inline]
            pub(crate) fn is_lock_free() -> bool {
                has_kuser_cmpxchg64()
            }
            #[inline]
            pub(crate) const fn is_always_lock_free() -> bool {
                false
            }

            #[inline]
            pub(crate) fn get_mut(&mut self) -> &mut $int_type {
                // SAFETY: the mutable reference guarantees unique ownership.
                // (UnsafeCell::get_mut requires Rust 1.50)
                unsafe { &mut *self.v.get() }
            }

            #[inline]
            pub(crate) fn into_inner(self) -> $int_type {
                self.v.into_inner()
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn load(&self, order: Ordering) -> $int_type {
                crate::utils::assert_load_ordering(order);
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_load(self.v.get().cast::<u64>()) as $int_type }
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn store(&self, val: $int_type, order: Ordering) {
                crate::utils::assert_store_ordering(order);
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_store(self.v.get().cast::<u64>(), val as u64) }
            }

            #[inline]
            pub(crate) fn swap(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_swap(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn compare_exchange(
                &self,
                current: $int_type,
                new: $int_type,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$int_type, $int_type> {
                crate::utils::assert_compare_exchange_ordering(success, failure);
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe {
                    let (prev, ok) = atomic_compare_exchange(
                        self.v.get().cast::<u64>(),
                        current as u64,
                        new as u64,
                    );
                    if ok {
                        Ok(prev as $int_type)
                    } else {
                        Err(prev as $int_type)
                    }
                }
            }

            #[inline]
            #[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
            pub(crate) fn compare_exchange_weak(
                &self,
                current: $int_type,
                new: $int_type,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$int_type, $int_type> {
                self.compare_exchange(current, new, success, failure)
            }

            #[inline]
            pub(crate) fn fetch_add(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_add(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_sub(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_sub(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_and(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_and(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_nand(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_nand(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_or(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_or(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_xor(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_xor(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_max(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { $atomic_max(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_min(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { $atomic_min(self.v.get().cast::<u64>(), val as u64) as $int_type }
            }

            #[inline]
            pub(crate) fn fetch_not(&self, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_not(self.v.get().cast::<u64>()) as $int_type }
            }
            #[inline]
            pub(crate) fn not(&self, order: Ordering) {
                self.fetch_not(order);
            }

            #[inline]
            pub(crate) fn fetch_neg(&self, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by the kernel user helper or the lock
                // and the raw pointer passed in is valid because we got it from a reference.
                unsafe { atomic_neg(self.v.get().cast::<u64>()) as $int_type }
            }
            #[inline]
            pub(crate) fn neg(&self, order: Ordering) {
                self.fetch_neg(order);
            }

            #[inline]
            pub(crate) const fn as_ptr(&self) -> *mut $int_type {
                self.v.get()
            }
        }
    };
}

atomic64!(AtomicI64, i64, atomic_max, atomic_min);
atomic64!(AtomicU64, u64, atomic_umax, atomic_umin);

#[allow(
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::undocumented_unsafe_blocks,
    clippy::wildcard_imports
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuser_helper_version() {
        let version = __kuser_helper_version();
        assert!(version >= 5, "{:?}", version);
        assert_eq!(version, unsafe { (KUSER_HELPER_VERSION as *const i32).read() });
    }

    test_atomic_int!(i64);
    test_atomic_int!(u64);

    // load/store/swap implementation is not affected by signedness, so it is
    // enough to test only unsigned types.
    stress_test!(u64);
}
