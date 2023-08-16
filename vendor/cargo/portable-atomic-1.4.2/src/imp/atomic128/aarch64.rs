// Atomic{I,U}128 implementation on AArch64.
//
// There are a few ways to implement 128-bit atomic operations in AArch64.
//
// - LDXP/STXP loop (DW LL/SC)
// - CASP (DWCAS) added as FEAT_LSE (mandatory from armv8.1-a)
// - LDP/STP (DW load/store) if FEAT_LSE2 (optional from armv8.2-a, mandatory from armv8.4-a) is available
//
// If outline-atomics is not enabled and FEAT_LSE is not available at
// compile-time, we use LDXP/STXP loop.
// If outline-atomics is enabled and FEAT_LSE is not available at
// compile-time, we use CASP for CAS if FEAT_LSE is available
// at run-time, otherwise, use LDXP/STXP loop.
// If FEAT_LSE is available at compile-time, we use CASP for load/store/CAS/RMW.
// However, when portable_atomic_ll_sc_rmw cfg is set, use LDXP/STXP loop instead of CASP
// loop for RMW (by default, it is set on Apple hardware; see build script for details).
// If FEAT_LSE2 is available at compile-time, we use LDP/STP for load/store.
//
// Note: FEAT_LSE2 doesn't imply FEAT_LSE.
//
// Note that we do not separate LL and SC into separate functions, but handle
// them within a single asm block. This is because it is theoretically possible
// for the compiler to insert operations that might clear the reservation between
// LL and SC. Considering the type of operations we are providing and the fact
// that [progress64](https://github.com/ARM-software/progress64) uses such code,
// this is probably not a problem for aarch64, but it seems that aarch64 doesn't
// guarantee it and hexagon is the only architecture with hardware guarantees
// that such code works. See also:
//
// - https://yarchive.net/comp/linux/cmpxchg_ll_sc_portability.html
// - https://lists.llvm.org/pipermail/llvm-dev/2016-May/099490.html
// - https://lists.llvm.org/pipermail/llvm-dev/2018-June/123993.html
//
// Also, even when using a CAS loop to implement atomic RMW, include the loop itself
// in the asm block because it is more efficient for some codegen backends.
// https://github.com/rust-lang/compiler-builtins/issues/339#issuecomment-1191260474
//
// Note: On Miri and ThreadSanitizer which do not support inline assembly, we don't use
// this module and use intrinsics.rs instead.
//
// Refs:
// - ARM Compiler armasm User Guide
//   https://developer.arm.com/documentation/dui0801/latest
// - Arm A-profile A64 Instruction Set Architecture
//   https://developer.arm.com/documentation/ddi0602/latest
// - Arm Architecture Reference Manual for A-profile architecture
//   https://developer.arm.com/documentation/ddi0487/latest
// - atomic-maybe-uninit https://github.com/taiki-e/atomic-maybe-uninit
//
// Generated asm:
// - aarch64 https://godbolt.org/z/nds1nWbnq
// - aarch64 msvc https://godbolt.org/z/PTKdhbKqW
// - aarch64 (+lse) https://godbolt.org/z/5GzssfTKc
// - aarch64 msvc (+lse) https://godbolt.org/z/oYE87caM7
// - aarch64 (+lse,+lse2) https://godbolt.org/z/36dPjMbaG

include!("macros.rs");

// On musl with static linking, it seems that getauxval is not always available.
// See detect/auxv.rs for more.
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(test, not(any(target_feature = "lse", portable_atomic_target_feature = "lse"))))]
#[cfg(any(
    all(
        target_os = "linux",
        any(
            target_env = "gnu",
            all(any(target_env = "musl", target_env = "ohos"), not(target_feature = "crt-static")),
            portable_atomic_outline_atomics,
        ),
    ),
    target_os = "android",
    target_os = "freebsd",
))]
#[path = "detect/auxv.rs"]
mod detect;
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(test, not(any(target_feature = "lse", portable_atomic_target_feature = "lse"))))]
#[cfg(target_os = "openbsd")]
#[path = "detect/aarch64_aa64reg.rs"]
mod detect;
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(test, not(any(target_feature = "lse", portable_atomic_target_feature = "lse"))))]
#[cfg(target_os = "fuchsia")]
#[path = "detect/aarch64_fuchsia.rs"]
mod detect;
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(test, not(any(target_feature = "lse", portable_atomic_target_feature = "lse"))))]
#[cfg(target_os = "windows")]
#[path = "detect/aarch64_windows.rs"]
mod detect;

// test only
#[cfg(test)]
#[cfg(not(qemu))]
#[cfg(not(valgrind))]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
#[path = "detect/aarch64_aa64reg.rs"]
mod detect_aa64reg;
#[cfg(test)]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(target_os = "macos")]
#[path = "detect/aarch64_macos.rs"]
mod detect_macos;

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;
use core::sync::atomic::Ordering;

#[cfg(any(
    target_feature = "lse",
    portable_atomic_target_feature = "lse",
    not(portable_atomic_no_outline_atomics),
))]
macro_rules! debug_assert_lse {
    () => {
        #[cfg(all(
            not(portable_atomic_no_outline_atomics),
            any(
                all(
                    target_os = "linux",
                    any(
                        target_env = "gnu",
                        all(
                            any(target_env = "musl", target_env = "ohos"),
                            not(target_feature = "crt-static"),
                        ),
                        portable_atomic_outline_atomics,
                    ),
                ),
                target_os = "android",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "fuchsia",
                target_os = "windows",
            ),
        ))]
        #[cfg(not(any(target_feature = "lse", portable_atomic_target_feature = "lse")))]
        {
            debug_assert!(detect::detect().has_lse());
        }
    };
}

#[cfg(target_endian = "little")]
macro_rules! select_le_or_be {
    ($le:expr, $be:expr) => {
        $le
    };
}
#[cfg(target_endian = "big")]
macro_rules! select_le_or_be {
    ($le:expr, $be:expr) => {
        $be
    };
}

/// A 128-bit value represented as a pair of 64-bit values.
///
/// This type is `#[repr(C)]`, both fields have the same in-memory representation
/// and are plain old datatypes, so access to the fields is always safe.
#[derive(Clone, Copy)]
#[repr(C)]
union U128 {
    whole: u128,
    pair: Pair,
}
// A pair of 64-bit values in little-endian order (even on big-endian targets).
#[derive(Clone, Copy)]
#[repr(C)]
struct Pair {
    lo: u64,
    hi: u64,
}

macro_rules! atomic_rmw {
    ($op:ident, $order:ident) => {
        atomic_rmw!($op, $order, write = $order)
    };
    ($op:ident, $order:ident, write = $write:ident) => {
        match $order {
            Ordering::Relaxed => $op!("", "", ""),
            Ordering::Acquire => $op!("a", "", ""),
            Ordering::Release => $op!("", "l", ""),
            Ordering::AcqRel => $op!("a", "l", ""),
            // In MSVC environments, SeqCst stores/writes needs fences after writes.
            // https://reviews.llvm.org/D141748
            #[cfg(target_env = "msvc")]
            Ordering::SeqCst if $write == Ordering::SeqCst => $op!("a", "l", "dmb ish"),
            // AcqRel and SeqCst RMWs are equivalent in non-MSVC environments.
            Ordering::SeqCst => $op!("a", "l", ""),
            _ => unreachable!("{:?}", $order),
        }
    };
}

#[inline]
unsafe fn atomic_load(src: *mut u128, order: Ordering) -> u128 {
    #[cfg(any(target_feature = "lse2", portable_atomic_target_feature = "lse2"))]
    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantee that the CPU supports FEAT_LSE2.
    unsafe {
        atomic_load_ldp(src, order)
    }
    #[cfg(not(any(target_feature = "lse2", portable_atomic_target_feature = "lse2")))]
    {
        #[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
        // SAFETY: the caller must uphold the safety contract.
        // cfg guarantee that the CPU supports FEAT_LSE.
        unsafe {
            _atomic_load_casp(src, order)
        }
        #[cfg(not(any(target_feature = "lse", portable_atomic_target_feature = "lse")))]
        // SAFETY: the caller must uphold the safety contract.
        unsafe {
            _atomic_load_ldxp_stxp(src, order)
        }
    }
}
// If CPU supports FEAT_LSE2, LDP is single-copy atomic reads,
// otherwise it is two single-copy atomic reads.
// Refs: B2.2.1 of the Arm Architecture Reference Manual Armv8, for Armv8-A architecture profile
#[cfg(any(target_feature = "lse2", portable_atomic_target_feature = "lse2"))]
#[inline]
unsafe fn atomic_load_ldp(src: *mut u128, order: Ordering) -> u128 {
    debug_assert!(src as usize % 16 == 0);

    // SAFETY: the caller must guarantee that `dst` is valid for reads,
    // 16-byte aligned, that there are no concurrent non-atomic operations.
    //
    // Refs:
    // - LDP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/LDP--A64-
    unsafe {
        let (prev_lo, prev_hi);
        macro_rules! atomic_load_relaxed {
            ($acquire:tt $(, $readonly:tt)?) => {
                asm!(
                    "ldp {prev_lo}, {prev_hi}, [{src}]",
                    $acquire,
                    src = in(reg) ptr_reg!(src),
                    prev_hi = lateout(reg) prev_hi,
                    prev_lo = lateout(reg) prev_lo,
                    options(nostack, preserves_flags $(, $readonly)?),
                )
            };
        }
        match order {
            Ordering::Relaxed => atomic_load_relaxed!("", readonly),
            Ordering::Acquire => atomic_load_relaxed!("dmb ishld"),
            Ordering::SeqCst => {
                asm!(
                    // ldar (or dmb ishld) is required to prevent reordering with preceding stlxp.
                    // See https://gcc.gnu.org/bugzilla/show_bug.cgi?id=108891 for details.
                    "ldar {tmp}, [{src}]",
                    "ldp {prev_lo}, {prev_hi}, [{src}]",
                    "dmb ishld",
                    src = in(reg) ptr_reg!(src),
                    prev_hi = lateout(reg) prev_hi,
                    prev_lo = lateout(reg) prev_lo,
                    tmp = out(reg) _,
                    options(nostack, preserves_flags),
                );
            }
            _ => unreachable!("{:?}", order),
        }
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}
// Do not use _atomic_compare_exchange_casp because it needs extra MOV to implement load.
#[cfg(any(test, not(any(target_feature = "lse2", portable_atomic_target_feature = "lse2"))))]
#[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
#[inline]
unsafe fn _atomic_load_casp(src: *mut u128, order: Ordering) -> u128 {
    debug_assert!(src as usize % 16 == 0);
    debug_assert_lse!();

    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantee that the CPU supports FEAT_LSE.
    unsafe {
        let (prev_lo, prev_hi);
        macro_rules! atomic_load {
            ($acquire:tt, $release:tt) => {
                asm!(
                    concat!("casp", $acquire, $release, " x2, x3, x2, x3, [{src}]"),
                    src = in(reg) ptr_reg!(src),
                    // must be allocated to even/odd register pair
                    inout("x2") 0_u64 => prev_lo,
                    inout("x3") 0_u64 => prev_hi,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            Ordering::Relaxed => atomic_load!("", ""),
            Ordering::Acquire => atomic_load!("a", ""),
            Ordering::SeqCst => atomic_load!("a", "l"),
            _ => unreachable!("{:?}", order),
        }
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}
#[cfg(any(
    test,
    all(
        not(any(target_feature = "lse2", portable_atomic_target_feature = "lse2")),
        not(any(target_feature = "lse", portable_atomic_target_feature = "lse")),
    ),
))]
#[inline]
unsafe fn _atomic_load_ldxp_stxp(src: *mut u128, order: Ordering) -> u128 {
    debug_assert!(src as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let (mut prev_lo, mut prev_hi);
        macro_rules! atomic_load {
            ($acquire:tt, $release:tt) => {
                asm!(
                    "2:",
                        concat!("ld", $acquire, "xp {prev_lo}, {prev_hi}, [{src}]"),
                        concat!("st", $release, "xp {r:w}, {prev_lo}, {prev_hi}, [{src}]"),
                        // 0 if the store was successful, 1 if no store was performed
                        "cbnz {r:w}, 2b",
                    src = in(reg) ptr_reg!(src),
                    prev_lo = out(reg) prev_lo,
                    prev_hi = out(reg) prev_hi,
                    r = out(reg) _,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            Ordering::Relaxed => atomic_load!("", ""),
            Ordering::Acquire => atomic_load!("a", ""),
            Ordering::SeqCst => atomic_load!("a", "l"),
            _ => unreachable!("{:?}", order),
        }
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}

#[inline]
unsafe fn atomic_store(dst: *mut u128, val: u128, order: Ordering) {
    #[cfg(any(target_feature = "lse2", portable_atomic_target_feature = "lse2"))]
    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantee that the CPU supports FEAT_LSE2.
    unsafe {
        atomic_store_stp(dst, val, order);
    }
    #[cfg(not(any(target_feature = "lse2", portable_atomic_target_feature = "lse2")))]
    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        atomic_swap(dst, val, order);
    }
}
// If CPU supports FEAT_LSE2, STP is single-copy atomic writes,
// otherwise it is two single-copy atomic writes.
// Refs: B2.2.1 of the Arm Architecture Reference Manual Armv8, for Armv8-A architecture profile
#[cfg(any(target_feature = "lse2", portable_atomic_target_feature = "lse2"))]
#[inline]
unsafe fn atomic_store_stp(dst: *mut u128, val: u128, order: Ordering) {
    debug_assert!(dst as usize % 16 == 0);

    // SAFETY: the caller must guarantee that `dst` is valid for writes,
    // 16-byte aligned, that there are no concurrent non-atomic operations.
    //
    // Refs:
    // - STP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/STP--A64-
    unsafe {
        let val = U128 { whole: val };
        macro_rules! atomic_store {
            ($acquire:tt, $release:tt) => {
                asm!(
                    $release,
                    "stp {val_lo}, {val_hi}, [{dst}]",
                    $acquire,
                    dst = in(reg) ptr_reg!(dst),
                    val_lo = in(reg) val.pair.lo,
                    val_hi = in(reg) val.pair.hi,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            Ordering::Relaxed => atomic_store!("", ""),
            Ordering::Release => atomic_store!("", "dmb ish"),
            Ordering::SeqCst => atomic_store!("dmb ish", "dmb ish"),
            _ => unreachable!("{:?}", order),
        }
    }
}

#[inline]
unsafe fn atomic_compare_exchange(
    dst: *mut u128,
    old: u128,
    new: u128,
    success: Ordering,
    failure: Ordering,
) -> Result<u128, u128> {
    #[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantee that the CPU supports FEAT_LSE.
    let res = unsafe { _atomic_compare_exchange_casp(dst, old, new, success, failure) };
    #[cfg(not(all(
        not(portable_atomic_no_outline_atomics),
        any(
            all(
                target_os = "linux",
                any(
                    target_env = "gnu",
                    all(
                        any(target_env = "musl", target_env = "ohos"),
                        not(target_feature = "crt-static"),
                    ),
                    portable_atomic_outline_atomics,
                ),
            ),
            target_os = "android",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "fuchsia",
            target_os = "windows",
        ),
    )))]
    #[cfg(not(any(target_feature = "lse", portable_atomic_target_feature = "lse")))]
    // SAFETY: the caller must uphold the safety contract.
    let res = unsafe { _atomic_compare_exchange_ldxp_stxp(dst, old, new, success, failure) };
    #[cfg(all(
        not(portable_atomic_no_outline_atomics),
        any(
            all(
                target_os = "linux",
                any(
                    target_env = "gnu",
                    all(
                        any(target_env = "musl", target_env = "ohos"),
                        not(target_feature = "crt-static"),
                    ),
                    portable_atomic_outline_atomics,
                ),
            ),
            target_os = "android",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "fuchsia",
            target_os = "windows",
        ),
    ))]
    #[cfg(not(any(target_feature = "lse", portable_atomic_target_feature = "lse")))]
    let res = {
        fn_alias! {
            #[target_feature(enable = "lse")]
            unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128;
            atomic_compare_exchange_casp_relaxed
                = _atomic_compare_exchange_casp(Ordering::Relaxed, Ordering::Relaxed);
            atomic_compare_exchange_casp_acquire
                = _atomic_compare_exchange_casp(Ordering::Acquire, Ordering::Acquire);
            atomic_compare_exchange_casp_release
                = _atomic_compare_exchange_casp(Ordering::Release, Ordering::Relaxed);
            atomic_compare_exchange_casp_acqrel
                = _atomic_compare_exchange_casp(Ordering::AcqRel, Ordering::Acquire);
            // AcqRel and SeqCst RMWs are equivalent in non-MSVC environments.
            #[cfg(target_env = "msvc")]
            atomic_compare_exchange_casp_seqcst
                = _atomic_compare_exchange_casp(Ordering::SeqCst, Ordering::SeqCst);
        }
        fn_alias! {
            unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128;
            atomic_compare_exchange_ldxp_stxp_relaxed
                = _atomic_compare_exchange_ldxp_stxp(Ordering::Relaxed, Ordering::Relaxed);
            atomic_compare_exchange_ldxp_stxp_acquire
                = _atomic_compare_exchange_ldxp_stxp(Ordering::Acquire, Ordering::Acquire);
            atomic_compare_exchange_ldxp_stxp_release
                = _atomic_compare_exchange_ldxp_stxp(Ordering::Release, Ordering::Relaxed);
            atomic_compare_exchange_ldxp_stxp_acqrel
                = _atomic_compare_exchange_ldxp_stxp(Ordering::AcqRel, Ordering::Acquire);
            // AcqRel and SeqCst RMWs are equivalent in non-MSVC environments.
            #[cfg(target_env = "msvc")]
            atomic_compare_exchange_ldxp_stxp_seqcst
                = _atomic_compare_exchange_ldxp_stxp(Ordering::SeqCst, Ordering::SeqCst);
        }
        // SAFETY: the caller must guarantee that `dst` is valid for both writes and
        // reads, 16-byte aligned, that there are no concurrent non-atomic operations,
        // and we've checked if FEAT_LSE is available.
        unsafe {
            let success = crate::utils::upgrade_success_ordering(success, failure);
            match success {
                Ordering::Relaxed => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_relaxed
                        } else {
                            atomic_compare_exchange_ldxp_stxp_relaxed
                        }
                    })
                }
                Ordering::Acquire => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_acquire
                        } else {
                            atomic_compare_exchange_ldxp_stxp_acquire
                        }
                    })
                }
                Ordering::Release => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_release
                        } else {
                            atomic_compare_exchange_ldxp_stxp_release
                        }
                    })
                }
                // AcqRel and SeqCst RMWs are equivalent in both implementations in non-MSVC environments.
                #[cfg(not(target_env = "msvc"))]
                Ordering::AcqRel | Ordering::SeqCst => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_acqrel
                        } else {
                            atomic_compare_exchange_ldxp_stxp_acqrel
                        }
                    })
                }
                #[cfg(target_env = "msvc")]
                Ordering::AcqRel => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_acqrel
                        } else {
                            atomic_compare_exchange_ldxp_stxp_acqrel
                        }
                    })
                }
                #[cfg(target_env = "msvc")]
                Ordering::SeqCst => {
                    ifunc!(unsafe fn(dst: *mut u128, old: u128, new: u128) -> u128 {
                        if detect::detect().has_lse() {
                            atomic_compare_exchange_casp_seqcst
                        } else {
                            atomic_compare_exchange_ldxp_stxp_seqcst
                        }
                    })
                }
                _ => unreachable!("{:?}", success),
            }
        }
    };
    if res == old {
        Ok(res)
    } else {
        Err(res)
    }
}
#[cfg(any(
    target_feature = "lse",
    portable_atomic_target_feature = "lse",
    not(portable_atomic_no_outline_atomics),
))]
#[cfg_attr(
    not(any(target_feature = "lse", portable_atomic_target_feature = "lse")),
    target_feature(enable = "lse")
)]
#[inline]
unsafe fn _atomic_compare_exchange_casp(
    dst: *mut u128,
    old: u128,
    new: u128,
    success: Ordering,
    failure: Ordering,
) -> u128 {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_lse!();
    let order = crate::utils::upgrade_success_ordering(success, failure);

    // SAFETY: the caller must guarantee that `dst` is valid for both writes and
    // reads, 16-byte aligned, that there are no concurrent non-atomic operations,
    // and the CPU supports FEAT_LSE.
    //
    // Refs:
    // - https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/CASPA--CASPAL--CASP--CASPL--CASPAL--CASP--CASPL--A64-
    // - https://developer.arm.com/documentation/ddi0602/2023-06/Base-Instructions/CASP--CASPA--CASPAL--CASPL--Compare-and-Swap-Pair-of-words-or-doublewords-in-memory-
    unsafe {
        let old = U128 { whole: old };
        let new = U128 { whole: new };
        let (prev_lo, prev_hi);
        macro_rules! cmpxchg {
            ($acquire:tt, $release:tt, $fence:tt) => {
                asm!(
                    concat!("casp", $acquire, $release, " x6, x7, x4, x5, [{dst}]"),
                    $fence,
                    dst = in(reg) ptr_reg!(dst),
                    // must be allocated to even/odd register pair
                    inout("x6") old.pair.lo => prev_lo,
                    inout("x7") old.pair.hi => prev_hi,
                    // must be allocated to even/odd register pair
                    in("x4") new.pair.lo,
                    in("x5") new.pair.hi,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw!(cmpxchg, order, write = success);
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}
#[cfg(any(test, not(any(target_feature = "lse", portable_atomic_target_feature = "lse"))))]
#[inline]
unsafe fn _atomic_compare_exchange_ldxp_stxp(
    dst: *mut u128,
    old: u128,
    new: u128,
    success: Ordering,
    failure: Ordering,
) -> u128 {
    debug_assert!(dst as usize % 16 == 0);
    let order = crate::utils::upgrade_success_ordering(success, failure);

    // SAFETY: the caller must guarantee that `dst` is valid for both writes and
    // reads, 16-byte aligned, and that there are no concurrent non-atomic operations.
    //
    // Refs:
    // - LDXP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/LDXP--A64-
    // - LDAXP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/LDAXP--A64-
    // - STXP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/STXP--A64-
    // - STLXP: https://developer.arm.com/documentation/dui0801/l/A64-Data-Transfer-Instructions/STLXP--A64-
    //
    // Note: Load-Exclusive pair (by itself) does not guarantee atomicity; to complete an atomic
    // operation (even load/store), a corresponding Store-Exclusive pair must succeed.
    // See Arm Architecture Reference Manual for A-profile architecture
    // Section B2.2.1 "Requirements for single-copy atomicity", and
    // Section B2.9 "Synchronization and semaphores" for more.
    unsafe {
        let old = U128 { whole: old };
        let new = U128 { whole: new };
        let (mut prev_lo, mut prev_hi);
        macro_rules! cmpxchg {
            ($acquire:tt, $release:tt, $fence:tt) => {
                asm!(
                    "2:",
                        concat!("ld", $acquire, "xp {out_lo}, {out_hi}, [{dst}]"),
                        "cmp {out_lo}, {old_lo}",
                        "cset {r:w}, ne",
                        "cmp {out_hi}, {old_hi}",
                        "cinc {r:w}, {r:w}, ne",
                        "cbz {r:w}, 3f",
                        concat!("st", $release, "xp {r:w}, {out_lo}, {out_hi}, [{dst}]"),
                        // 0 if the store was successful, 1 if no store was performed
                        "cbnz {r:w}, 2b",
                        "b 4f",
                    "3:",
                        concat!("st", $release, "xp {r:w}, {new_lo}, {new_hi}, [{dst}]"),
                        // 0 if the store was successful, 1 if no store was performed
                        "cbnz {r:w}, 2b",
                    "4:",
                    $fence,
                    dst = in(reg) ptr_reg!(dst),
                    old_lo = in(reg) old.pair.lo,
                    old_hi = in(reg) old.pair.hi,
                    new_lo = in(reg) new.pair.lo,
                    new_hi = in(reg) new.pair.hi,
                    out_lo = out(reg) prev_lo,
                    out_hi = out(reg) prev_hi,
                    r = out(reg) _,
                    // Do not use `preserves_flags` because CMP modifies the condition flags.
                    options(nostack),
                )
            };
        }
        atomic_rmw!(cmpxchg, order, write = success);
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}

// casp is always strong, and ldxp requires a corresponding (succeed) stxp for
// its atomicity (see code comment in _atomic_compare_exchange_ldxp_stxp).
// (i.e., aarch64 doesn't have 128-bit weak CAS)
use self::atomic_compare_exchange as atomic_compare_exchange_weak;

// If FEAT_LSE is available at compile-time and portable_atomic_ll_sc_rmw cfg is not set,
// we use CAS-based atomic RMW.
#[cfg(all(
    any(target_feature = "lse", portable_atomic_target_feature = "lse"),
    not(portable_atomic_ll_sc_rmw),
))]
use _atomic_swap_casp as atomic_swap;
#[cfg(not(all(
    any(target_feature = "lse", portable_atomic_target_feature = "lse"),
    not(portable_atomic_ll_sc_rmw),
)))]
use _atomic_swap_ldxp_stxp as atomic_swap;
// Do not use atomic_rmw_cas_3 because it needs extra MOV to implement swap.
#[cfg(any(
    test,
    all(
        any(target_feature = "lse", portable_atomic_target_feature = "lse"),
        not(portable_atomic_ll_sc_rmw),
    )
))]
#[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
#[inline]
unsafe fn _atomic_swap_casp(dst: *mut u128, val: u128, order: Ordering) -> u128 {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_lse!();

    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantee that the CPU supports FEAT_LSE.
    unsafe {
        let val = U128 { whole: val };
        let (mut prev_lo, mut prev_hi);
        macro_rules! swap {
            ($acquire:tt, $release:tt, $fence:tt) => {
                asm!(
                    // If FEAT_LSE2 is not supported, this works like byte-wise atomic.
                    // This is not single-copy atomic reads, but this is ok because subsequent
                    // CAS will check for consistency.
                    "ldp x4, x5, [{dst}]",
                    "2:",
                        // casp writes the current value to the first register pair,
                        // so copy the `out`'s value for later comparison.
                        "mov {tmp_lo}, x4",
                        "mov {tmp_hi}, x5",
                        concat!("casp", $acquire, $release, " x4, x5, x2, x3, [{dst}]"),
                        "cmp {tmp_hi}, x5",
                        "ccmp {tmp_lo}, x4, #0, eq",
                        "b.ne 2b",
                    $fence,
                    dst = in(reg) ptr_reg!(dst),
                    tmp_lo = out(reg) _,
                    tmp_hi = out(reg) _,
                    // must be allocated to even/odd register pair
                    out("x4") prev_lo,
                    out("x5") prev_hi,
                    // must be allocated to even/odd register pair
                    in("x2") val.pair.lo,
                    in("x3") val.pair.hi,
                    // Do not use `preserves_flags` because CMP and CCMP modify the condition flags.
                    options(nostack),
                )
            };
        }
        atomic_rmw!(swap, order);
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}
// Do not use atomic_rmw_ll_sc_3 because it needs extra MOV to implement swap.
#[cfg(any(
    test,
    not(all(
        any(target_feature = "lse", portable_atomic_target_feature = "lse"),
        not(portable_atomic_ll_sc_rmw),
    ))
))]
#[inline]
unsafe fn _atomic_swap_ldxp_stxp(dst: *mut u128, val: u128, order: Ordering) -> u128 {
    debug_assert!(dst as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let val = U128 { whole: val };
        let (mut prev_lo, mut prev_hi);
        macro_rules! swap {
            ($acquire:tt, $release:tt, $fence:tt) => {
                asm!(
                    "2:",
                        concat!("ld", $acquire, "xp {prev_lo}, {prev_hi}, [{dst}]"),
                        concat!("st", $release, "xp {r:w}, {val_lo}, {val_hi}, [{dst}]"),
                        // 0 if the store was successful, 1 if no store was performed
                        "cbnz {r:w}, 2b",
                    $fence,
                    dst = in(reg) ptr_reg!(dst),
                    val_lo = in(reg) val.pair.lo,
                    val_hi = in(reg) val.pair.hi,
                    prev_lo = out(reg) prev_lo,
                    prev_hi = out(reg) prev_hi,
                    r = out(reg) _,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw!(swap, order);
        U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
    }
}

/// Atomic RMW by LL/SC loop (3 arguments)
/// `unsafe fn(dst: *mut u128, val: u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - val_lo/val_hi pair: val argument (read-only for `$op`)
/// - prev_lo/prev_hi pair: previous value loaded by ll (read-only for `$op`)
/// - new_lo/new_hi pair: new value that will to stored by sc
macro_rules! atomic_rmw_ll_sc_3 {
    ($name:ident as $reexport_name:ident $(($preserves_flags:tt))?, $($op:tt)*) => {
        // If FEAT_LSE is available at compile-time and portable_atomic_ll_sc_rmw cfg is not set,
        // we use CAS-based atomic RMW generated by atomic_rmw_cas_3! macro instead.
        #[cfg(not(all(
            any(target_feature = "lse", portable_atomic_target_feature = "lse"),
            not(portable_atomic_ll_sc_rmw),
        )))]
        use $name as $reexport_name;
        #[cfg(any(
            test,
            not(all(
                any(target_feature = "lse", portable_atomic_target_feature = "lse"),
                not(portable_atomic_ll_sc_rmw),
            ))
        ))]
        #[inline]
        unsafe fn $name(dst: *mut u128, val: u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                let val = U128 { whole: val };
                let (mut prev_lo, mut prev_hi);
                macro_rules! op {
                    ($acquire:tt, $release:tt, $fence:tt) => {
                        asm!(
                            "2:",
                                concat!("ld", $acquire, "xp {prev_lo}, {prev_hi}, [{dst}]"),
                                $($op)*
                                concat!("st", $release, "xp {r:w}, {new_lo}, {new_hi}, [{dst}]"),
                                // 0 if the store was successful, 1 if no store was performed
                                "cbnz {r:w}, 2b",
                            $fence,
                            dst = in(reg) ptr_reg!(dst),
                            val_lo = in(reg) val.pair.lo,
                            val_hi = in(reg) val.pair.hi,
                            prev_lo = out(reg) prev_lo,
                            prev_hi = out(reg) prev_hi,
                            new_lo = out(reg) _,
                            new_hi = out(reg) _,
                            r = out(reg) _,
                            options(nostack $(, $preserves_flags)?),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
            }
        }
    };
}
/// Atomic RMW by CAS loop (3 arguments)
/// `unsafe fn(dst: *mut u128, val: u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - val_lo/val_hi pair: val argument (read-only for `$op`)
/// - x6/x7 pair: previous value loaded (read-only for `$op`)
/// - x4/x5 pair: new value that will to stored
macro_rules! atomic_rmw_cas_3 {
    ($name:ident as $reexport_name:ident, $($op:tt)*) => {
        // If FEAT_LSE is not available at compile-time or portable_atomic_ll_sc_rmw cfg is set,
        // we use LL/SC-based atomic RMW generated by atomic_rmw_ll_sc_3! macro instead.
        #[cfg(all(
            any(target_feature = "lse", portable_atomic_target_feature = "lse"),
            not(portable_atomic_ll_sc_rmw),
        ))]
        use $name as $reexport_name;
        #[cfg(any(
            test,
            all(
                any(target_feature = "lse", portable_atomic_target_feature = "lse"),
                not(portable_atomic_ll_sc_rmw),
            )
        ))]
        #[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
        #[inline]
        unsafe fn $name(dst: *mut u128, val: u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            debug_assert_lse!();
            // SAFETY: the caller must uphold the safety contract.
            // cfg guarantee that the CPU supports FEAT_LSE.
            unsafe {
                let val = U128 { whole: val };
                let (mut prev_lo, mut prev_hi);
                macro_rules! op {
                    ($acquire:tt, $release:tt, $fence:tt) => {
                        asm!(
                            // If FEAT_LSE2 is not supported, this works like byte-wise atomic.
                            // This is not single-copy atomic reads, but this is ok because subsequent
                            // CAS will check for consistency.
                            "ldp x6, x7, [{dst}]",
                            "2:",
                                // casp writes the current value to the first register pair,
                                // so copy the `out`'s value for later comparison.
                                "mov {tmp_lo}, x6",
                                "mov {tmp_hi}, x7",
                                $($op)*
                                concat!("casp", $acquire, $release, " x6, x7, x4, x5, [{dst}]"),
                                "cmp {tmp_hi}, x7",
                                "ccmp {tmp_lo}, x6, #0, eq",
                                "b.ne 2b",
                            $fence,
                            dst = in(reg) ptr_reg!(dst),
                            val_lo = in(reg) val.pair.lo,
                            val_hi = in(reg) val.pair.hi,
                            tmp_lo = out(reg) _,
                            tmp_hi = out(reg) _,
                            // must be allocated to even/odd register pair
                            out("x6") prev_lo,
                            out("x7") prev_hi,
                            // must be allocated to even/odd register pair
                            out("x4") _,
                            out("x5") _,
                            // Do not use `preserves_flags` because CMP and CCMP modify the condition flags.
                            options(nostack),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
            }
        }
    };
}

/// Atomic RMW by LL/SC loop (2 arguments)
/// `unsafe fn(dst: *mut u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - prev_lo/prev_hi pair: previous value loaded by ll (read-only for `$op`)
/// - new_lo/new_hi pair: new value that will to stored by sc
macro_rules! atomic_rmw_ll_sc_2 {
    ($name:ident as $reexport_name:ident $(($preserves_flags:tt))?, $($op:tt)*) => {
        // If FEAT_LSE is available at compile-time and portable_atomic_ll_sc_rmw cfg is not set,
        // we use CAS-based atomic RMW generated by atomic_rmw_cas_2! macro instead.
        #[cfg(not(all(
            any(target_feature = "lse", portable_atomic_target_feature = "lse"),
            not(portable_atomic_ll_sc_rmw),
        )))]
        use $name as $reexport_name;
        #[cfg(any(
            test,
            not(all(
                any(target_feature = "lse", portable_atomic_target_feature = "lse"),
                not(portable_atomic_ll_sc_rmw),
            ))
        ))]
        #[inline]
        unsafe fn $name(dst: *mut u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                let (mut prev_lo, mut prev_hi);
                macro_rules! op {
                    ($acquire:tt, $release:tt, $fence:tt) => {
                        asm!(
                            "2:",
                                concat!("ld", $acquire, "xp {prev_lo}, {prev_hi}, [{dst}]"),
                                $($op)*
                                concat!("st", $release, "xp {r:w}, {new_lo}, {new_hi}, [{dst}]"),
                                // 0 if the store was successful, 1 if no store was performed
                                "cbnz {r:w}, 2b",
                            $fence,
                            dst = in(reg) ptr_reg!(dst),
                            prev_lo = out(reg) prev_lo,
                            prev_hi = out(reg) prev_hi,
                            new_lo = out(reg) _,
                            new_hi = out(reg) _,
                            r = out(reg) _,
                            options(nostack $(, $preserves_flags)?),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
            }
        }
    };
}
/// Atomic RMW by CAS loop (2 arguments)
/// `unsafe fn(dst: *mut u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - x6/x7 pair: previous value loaded (read-only for `$op`)
/// - x4/x5 pair: new value that will to stored
macro_rules! atomic_rmw_cas_2 {
    ($name:ident as $reexport_name:ident, $($op:tt)*) => {
        // If FEAT_LSE is not available at compile-time or portable_atomic_ll_sc_rmw cfg is set,
        // we use LL/SC-based atomic RMW generated by atomic_rmw_ll_sc_3! macro instead.
        #[cfg(all(
            any(target_feature = "lse", portable_atomic_target_feature = "lse"),
            not(portable_atomic_ll_sc_rmw),
        ))]
        use $name as $reexport_name;
        #[cfg(any(
            test,
            all(
                any(target_feature = "lse", portable_atomic_target_feature = "lse"),
                not(portable_atomic_ll_sc_rmw),
            )
        ))]
        #[cfg(any(target_feature = "lse", portable_atomic_target_feature = "lse"))]
        #[inline]
        unsafe fn $name(dst: *mut u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            debug_assert_lse!();
            // SAFETY: the caller must uphold the safety contract.
            // cfg guarantee that the CPU supports FEAT_LSE.
            unsafe {
                let (mut prev_lo, mut prev_hi);
                macro_rules! op {
                    ($acquire:tt, $release:tt, $fence:tt) => {
                        asm!(
                            // If FEAT_LSE2 is not supported, this works like byte-wise atomic.
                            // This is not single-copy atomic reads, but this is ok because subsequent
                            // CAS will check for consistency.
                            "ldp x6, x7, [{dst}]",
                            "2:",
                                // casp writes the current value to the first register pair,
                                // so copy the `out`'s value for later comparison.
                                "mov {tmp_lo}, x6",
                                "mov {tmp_hi}, x7",
                                $($op)*
                                concat!("casp", $acquire, $release, " x6, x7, x4, x5, [{dst}]"),
                                "cmp {tmp_hi}, x7",
                                "ccmp {tmp_lo}, x6, #0, eq",
                                "b.ne 2b",
                            $fence,
                            dst = in(reg) ptr_reg!(dst),
                            tmp_lo = out(reg) _,
                            tmp_hi = out(reg) _,
                            // must be allocated to even/odd register pair
                            out("x6") prev_lo,
                            out("x7") prev_hi,
                            // must be allocated to even/odd register pair
                            out("x4") _,
                            out("x5") _,
                            // Do not use `preserves_flags` because CMP and CCMP modify the condition flags.
                            options(nostack),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { lo: prev_lo, hi: prev_hi } }.whole
            }
        }
    };
}

// Do not use `preserves_flags` because ADDS modifies the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_add_ldxp_stxp as atomic_add,
    select_le_or_be!("adds {new_lo}, {prev_lo}, {val_lo}", "adds {new_hi}, {prev_hi}, {val_hi}"),
    select_le_or_be!("adc {new_hi}, {prev_hi}, {val_hi}", "adc {new_lo}, {prev_lo}, {val_lo}"),
}
atomic_rmw_cas_3! {
    _atomic_add_casp as atomic_add,
    select_le_or_be!("adds x4, x6, {val_lo}", "adds x5, x7, {val_hi}"),
    select_le_or_be!("adc x5, x7, {val_hi}", "adc x4, x6, {val_lo}"),
}

// Do not use `preserves_flags` because SUBS modifies the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_sub_ldxp_stxp as atomic_sub,
    select_le_or_be!("subs {new_lo}, {prev_lo}, {val_lo}", "subs {new_hi}, {prev_hi}, {val_hi}"),
    select_le_or_be!("sbc {new_hi}, {prev_hi}, {val_hi}", "sbc {new_lo}, {prev_lo}, {val_lo}"),
}
atomic_rmw_cas_3! {
    _atomic_sub_casp as atomic_sub,
    select_le_or_be!("subs x4, x6, {val_lo}", "subs x5, x7, {val_hi}"),
    select_le_or_be!("sbc x5, x7, {val_hi}", "sbc x4, x6, {val_lo}"),
}

atomic_rmw_ll_sc_3! {
    _atomic_and_ldxp_stxp as atomic_and (preserves_flags),
    "and {new_lo}, {prev_lo}, {val_lo}",
    "and {new_hi}, {prev_hi}, {val_hi}",
}
atomic_rmw_cas_3! {
    _atomic_and_casp as atomic_and,
    "and x4, x6, {val_lo}",
    "and x5, x7, {val_hi}",
}

atomic_rmw_ll_sc_3! {
    _atomic_nand_ldxp_stxp as atomic_nand (preserves_flags),
    "and {new_lo}, {prev_lo}, {val_lo}",
    "mvn {new_lo}, {new_lo}",
    "and {new_hi}, {prev_hi}, {val_hi}",
    "mvn {new_hi}, {new_hi}",
}
atomic_rmw_cas_3! {
    _atomic_nand_casp as atomic_nand,
    "and x4, x6, {val_lo}",
    "mvn x4, x4",
    "and x5, x7, {val_hi}",
    "mvn x5, x5",
}

atomic_rmw_ll_sc_3! {
    _atomic_or_ldxp_stxp as atomic_or (preserves_flags),
    "orr {new_lo}, {prev_lo}, {val_lo}",
    "orr {new_hi}, {prev_hi}, {val_hi}",
}
atomic_rmw_cas_3! {
    _atomic_or_casp as atomic_or,
    "orr x4, x6, {val_lo}",
    "orr x5, x7, {val_hi}",
}

atomic_rmw_ll_sc_3! {
    _atomic_xor_ldxp_stxp as atomic_xor (preserves_flags),
    "eor {new_lo}, {prev_lo}, {val_lo}",
    "eor {new_hi}, {prev_hi}, {val_hi}",
}
atomic_rmw_cas_3! {
    _atomic_xor_casp as atomic_xor,
    "eor x4, x6, {val_lo}",
    "eor x5, x7, {val_hi}",
}

atomic_rmw_ll_sc_2! {
    _atomic_not_ldxp_stxp as atomic_not (preserves_flags),
    "mvn {new_lo}, {prev_lo}",
    "mvn {new_hi}, {prev_hi}",
}
atomic_rmw_cas_2! {
    _atomic_not_casp as atomic_not,
    "mvn x4, x6",
    "mvn x5, x7",
}

// Do not use `preserves_flags` because NEGS modifies the condition flags.
atomic_rmw_ll_sc_2! {
    _atomic_neg_ldxp_stxp as atomic_neg,
    select_le_or_be!("negs {new_lo}, {prev_lo}", "negs {new_hi}, {prev_hi}"),
    select_le_or_be!("ngc {new_hi}, {prev_hi}", "ngc {new_lo}, {prev_lo}"),
}
atomic_rmw_cas_2! {
    _atomic_neg_casp as atomic_neg,
    select_le_or_be!("negs x4, x6", "negs x5, x7"),
    select_le_or_be!("ngc x5, x7", "ngc x4, x6"),
}

// Do not use `preserves_flags` because CMP and SBCS modify the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_max_ldxp_stxp as atomic_max,
    select_le_or_be!("cmp {val_lo}, {prev_lo}", "cmp {val_hi}, {prev_hi}"),
    select_le_or_be!("sbcs xzr, {val_hi}, {prev_hi}", "sbcs xzr, {val_lo}, {prev_lo}"),
    "csel {new_hi}, {prev_hi}, {val_hi}, lt", // select hi 64-bit
    "csel {new_lo}, {prev_lo}, {val_lo}, lt", // select lo 64-bit
}
atomic_rmw_cas_3! {
    _atomic_max_casp as atomic_max,
    select_le_or_be!("cmp {val_lo}, x6", "cmp {val_hi}, x7"),
    select_le_or_be!("sbcs xzr, {val_hi}, x7", "sbcs xzr, {val_lo}, x6"),
    "csel x5, x7, {val_hi}, lt", // select hi 64-bit
    "csel x4, x6, {val_lo}, lt", // select lo 64-bit
}

// Do not use `preserves_flags` because CMP and SBCS modify the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_umax_ldxp_stxp as atomic_umax,
    select_le_or_be!("cmp {val_lo}, {prev_lo}", "cmp {val_hi}, {prev_hi}"),
    select_le_or_be!("sbcs xzr, {val_hi}, {prev_hi}", "sbcs xzr, {val_lo}, {prev_lo}"),
    "csel {new_hi}, {prev_hi}, {val_hi}, lo", // select hi 64-bit
    "csel {new_lo}, {prev_lo}, {val_lo}, lo", // select lo 64-bit
}
atomic_rmw_cas_3! {
    _atomic_umax_casp as atomic_umax,
    select_le_or_be!("cmp {val_lo}, x6", "cmp {val_hi}, x7"),
    select_le_or_be!("sbcs xzr, {val_hi}, x7", "sbcs xzr, {val_lo}, x6"),
    "csel x5, x7, {val_hi}, lo", // select hi 64-bit
    "csel x4, x6, {val_lo}, lo", // select lo 64-bit
}

// Do not use `preserves_flags` because CMP and SBCS modify the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_min_ldxp_stxp as atomic_min,
    select_le_or_be!("cmp {val_lo}, {prev_lo}", "cmp {val_hi}, {prev_hi}"),
    select_le_or_be!("sbcs xzr, {val_hi}, {prev_hi}", "sbcs xzr, {val_lo}, {prev_lo}"),
    "csel {new_hi}, {prev_hi}, {val_hi}, ge", // select hi 64-bit
    "csel {new_lo}, {prev_lo}, {val_lo}, ge", // select lo 64-bit
}
atomic_rmw_cas_3! {
    _atomic_min_casp as atomic_min,
    select_le_or_be!("cmp {val_lo}, x6", "cmp {val_hi}, x7"),
    select_le_or_be!("sbcs xzr, {val_hi}, x7", "sbcs xzr, {val_lo}, x6"),
    "csel x5, x7, {val_hi}, ge", // select hi 64-bit
    "csel x4, x6, {val_lo}, ge", // select lo 64-bit
}

// Do not use `preserves_flags` because CMP and SBCS modify the condition flags.
atomic_rmw_ll_sc_3! {
    _atomic_umin_ldxp_stxp as atomic_umin,
    select_le_or_be!("cmp {val_lo}, {prev_lo}", "cmp {val_hi}, {prev_hi}"),
    select_le_or_be!("sbcs xzr, {val_hi}, {prev_hi}", "sbcs xzr, {val_lo}, {prev_lo}"),
    "csel {new_hi}, {prev_hi}, {val_hi}, hs", // select hi 64-bit
    "csel {new_lo}, {prev_lo}, {val_lo}, hs", // select lo 64-bit
}
atomic_rmw_cas_3! {
    _atomic_umin_casp as atomic_umin,
    select_le_or_be!("cmp {val_lo}, x6", "cmp {val_hi}, x7"),
    select_le_or_be!("sbcs xzr, {val_hi}, x7", "sbcs xzr, {val_lo}, x6"),
    "csel x5, x7, {val_hi}, hs", // select hi 64-bit
    "csel x4, x6, {val_lo}, hs", // select lo 64-bit
}

#[inline]
const fn is_lock_free() -> bool {
    IS_ALWAYS_LOCK_FREE
}
const IS_ALWAYS_LOCK_FREE: bool = true;

atomic128!(AtomicI128, i128, atomic_max, atomic_min);
atomic128!(AtomicU128, u128, atomic_umax, atomic_umin);

#[cfg(test)]
mod tests {
    use super::*;

    test_atomic_int!(i128);
    test_atomic_int!(u128);

    // load/store/swap implementation is not affected by signedness, so it is
    // enough to test only unsigned types.
    stress_test!(u128);
}
