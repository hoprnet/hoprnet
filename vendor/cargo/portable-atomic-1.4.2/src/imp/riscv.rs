// Atomic load/store implementation on RISC-V.
//
// Refs:
// - "Mappings from C/C++ primitives to RISC-V primitives." table in RISC-V Instruction Set Manual:
//   https://five-embeddev.com/riscv-isa-manual/latest/memory.html#sec:memory:porting
// - atomic-maybe-uninit https://github.com/taiki-e/atomic-maybe-uninit
//
// Generated asm:
// - riscv64gc https://godbolt.org/z/hx4Krb91h

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;
use core::{cell::UnsafeCell, sync::atomic::Ordering};

macro_rules! atomic {
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

atomic!(AtomicI8, i8, "b");
atomic!(AtomicU8, u8, "b");
atomic!(AtomicI16, i16, "h");
atomic!(AtomicU16, u16, "h");
atomic!(AtomicI32, i32, "w");
atomic!(AtomicU32, u32, "w");
#[cfg(target_arch = "riscv64")]
atomic!(AtomicI64, i64, "d");
#[cfg(target_arch = "riscv64")]
atomic!(AtomicU64, u64, "d");
#[cfg(target_pointer_width = "32")]
atomic!(AtomicIsize, isize, "w");
#[cfg(target_pointer_width = "32")]
atomic!(AtomicUsize, usize, "w");
#[cfg(target_pointer_width = "32")]
atomic!([T] AtomicPtr, *mut T, "w");
#[cfg(target_pointer_width = "64")]
atomic!(AtomicIsize, isize, "d");
#[cfg(target_pointer_width = "64")]
atomic!(AtomicUsize, usize, "d");
#[cfg(target_pointer_width = "64")]
atomic!([T] AtomicPtr, *mut T, "d");

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
}
