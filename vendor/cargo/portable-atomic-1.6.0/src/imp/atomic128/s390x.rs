// SPDX-License-Identifier: Apache-2.0 OR MIT

// Atomic{I,U}128 implementation on s390x.
//
// s390x supports 128-bit atomic load/store/cmpxchg:
// https://github.com/llvm/llvm-project/commit/a11f63a952664f700f076fd754476a2b9eb158cc
//
// LLVM's minimal supported architecture level is z10:
// https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/lib/Target/SystemZ/SystemZProcessors.td)
// This does not appear to have changed since the current s390x backend was added in LLVM 3.3:
// https://github.com/llvm/llvm-project/commit/5f613dfd1f7edb0ae95d521b7107b582d9df5103#diff-cbaef692b3958312e80fd5507a7e2aff071f1acb086f10e8a96bc06a7bb289db
//
// Note: On Miri and ThreadSanitizer which do not support inline assembly, we don't use
// this module and use intrinsics.rs instead.
//
// Refs:
// - z/Architecture Principles of Operation https://publibfp.dhe.ibm.com/epubs/pdf/a227832d.pdf
// - z/Architecture Reference Summary https://www.ibm.com/support/pages/zarchitecture-reference-summary
// - atomic-maybe-uninit https://github.com/taiki-e/atomic-maybe-uninit
//
// Generated asm:
// - s390x https://godbolt.org/z/b11znnEh4
// - s390x (z196) https://godbolt.org/z/s5n9PGcv6
// - s390x (z15) https://godbolt.org/z/Wf49h7bPf

include!("macros.rs");

use core::{arch::asm, sync::atomic::Ordering};

use crate::utils::{Pair, U128};

// Use distinct operands on z196 or later, otherwise split to lgr and $op.
#[cfg(any(target_feature = "distinct-ops", portable_atomic_target_feature = "distinct-ops"))]
macro_rules! distinct_op {
    ($op:tt, $a0:tt, $a1:tt, $a2:tt) => {
        concat!($op, "k ", $a0, ", ", $a1, ", ", $a2)
    };
}
#[cfg(not(any(target_feature = "distinct-ops", portable_atomic_target_feature = "distinct-ops")))]
macro_rules! distinct_op {
    ($op:tt, $a0:tt, $a1:tt, $a2:tt) => {
        concat!("lgr ", $a0, ", ", $a1, "\n", $op, " ", $a0, ", ", $a2)
    };
}

// Use selgr$cond on z15 or later, otherwise split to locgr$cond and $op.
#[cfg(any(
    target_feature = "miscellaneous-extensions-3",
    portable_atomic_target_feature = "miscellaneous-extensions-3",
))]
#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
macro_rules! select_op {
    ($cond:tt, $a0:tt, $a1:tt, $a2:tt) => {
        concat!("selgr", $cond, " ", $a0, ", ", $a1, ", ", $a2)
    };
}
#[cfg(not(any(
    target_feature = "miscellaneous-extensions-3",
    portable_atomic_target_feature = "miscellaneous-extensions-3",
)))]
#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
macro_rules! select_op {
    ($cond:tt, $a0:tt, $a1:tt, $a2:tt) => {
        concat!("lgr ", $a0, ", ", $a2, "\n", "locgr", $cond, " ", $a0, ", ", $a1)
    };
}

#[inline]
unsafe fn atomic_load(src: *mut u128, _order: Ordering) -> u128 {
    debug_assert!(src as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        // atomic load is always SeqCst.
        let (out_hi, out_lo);
        asm!(
            "lpq %r0, 0({src})",
            src = in(reg) ptr_reg!(src),
            // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
            out("r0") out_hi,
            out("r1") out_lo,
            options(nostack, preserves_flags),
        );
        U128 { pair: Pair { hi: out_hi, lo: out_lo } }.whole
    }
}

#[inline]
unsafe fn atomic_store(dst: *mut u128, val: u128, order: Ordering) {
    debug_assert!(dst as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let val = U128 { whole: val };
        macro_rules! atomic_store {
            ($fence:tt) => {
                asm!(
                    "stpq %r0, 0({dst})",
                    $fence,
                    dst = in(reg) ptr_reg!(dst),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    in("r0") val.pair.hi,
                    in("r1") val.pair.lo,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            // Relaxed and Release stores are equivalent.
            Ordering::Relaxed | Ordering::Release => atomic_store!(""),
            // bcr 14,0 (fast-BCR-serialization) requires z196 or later.
            #[cfg(any(
                target_feature = "fast-serialization",
                portable_atomic_target_feature = "fast-serialization",
            ))]
            Ordering::SeqCst => atomic_store!("bcr 14, 0"),
            #[cfg(not(any(
                target_feature = "fast-serialization",
                portable_atomic_target_feature = "fast-serialization",
            )))]
            Ordering::SeqCst => atomic_store!("bcr 15, 0"),
            _ => unreachable!("{:?}", order),
        }
    }
}

#[inline]
unsafe fn atomic_compare_exchange(
    dst: *mut u128,
    old: u128,
    new: u128,
    _success: Ordering,
    _failure: Ordering,
) -> Result<u128, u128> {
    debug_assert!(dst as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    let prev = unsafe {
        // atomic CAS is always SeqCst.
        let old = U128 { whole: old };
        let new = U128 { whole: new };
        let (prev_hi, prev_lo);
        asm!(
            "cdsg %r0, %r12, 0({dst})",
            dst = in(reg) ptr_reg!(dst),
            // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
            inout("r0") old.pair.hi => prev_hi,
            inout("r1") old.pair.lo => prev_lo,
            in("r12") new.pair.hi,
            in("r13") new.pair.lo,
            // Do not use `preserves_flags` because CDSG modifies the condition code.
            options(nostack),
        );
        U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
    };
    if prev == old {
        Ok(prev)
    } else {
        Err(prev)
    }
}

// cdsg is always strong.
use atomic_compare_exchange as atomic_compare_exchange_weak;

#[cfg(not(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
)))]
#[inline(always)]
unsafe fn atomic_update<F>(dst: *mut u128, order: Ordering, mut f: F) -> u128
where
    F: FnMut(u128) -> u128,
{
    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        // This is a private function and all instances of `f` only operate on the value
        // loaded, so there is no need to synchronize the first load/failed CAS.
        let mut prev = atomic_load(dst, Ordering::Relaxed);
        loop {
            let next = f(prev);
            match atomic_compare_exchange_weak(dst, prev, next, order, Ordering::Relaxed) {
                Ok(x) => return x,
                Err(x) => prev = x,
            }
        }
    }
}

#[inline]
unsafe fn atomic_swap(dst: *mut u128, val: u128, _order: Ordering) -> u128 {
    debug_assert!(dst as usize % 16 == 0);

    // SAFETY: the caller must uphold the safety contract.
    //
    // We could use atomic_update here, but using an inline assembly allows omitting
    // the comparison of results and the storing/comparing of condition flags.
    //
    // Do not use atomic_rmw_cas_3 because it needs extra LGR to implement swap.
    unsafe {
        // atomic swap is always SeqCst.
        let val = U128 { whole: val };
        let (mut prev_hi, mut prev_lo);
        asm!(
            "lpq %r0, 0({dst})",
            "2:",
                "cdsg %r0, %r12, 0({dst})",
                "jl 2b",
            dst = in(reg) ptr_reg!(dst),
            // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
            out("r0") prev_hi,
            out("r1") prev_lo,
            in("r12") val.pair.hi,
            in("r13") val.pair.lo,
            // Do not use `preserves_flags` because CDSG modifies the condition code.
            options(nostack),
        );
        U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
    }
}

/// Atomic RMW by CAS loop (3 arguments)
/// `unsafe fn(dst: *mut u128, val: u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - val_hi/val_lo pair: val argument (read-only for `$op`)
/// - r0/r1 pair: previous value loaded (read-only for `$op`)
/// - r12/r13 pair: new value that will be stored
// We could use atomic_update here, but using an inline assembly allows omitting
// the comparison of results and the storing/comparing of condition flags.
macro_rules! atomic_rmw_cas_3 {
    ($name:ident, [$($reg:tt)*], $($op:tt)*) => {
        #[inline]
        unsafe fn $name(dst: *mut u128, val: u128, _order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                // atomic RMW is always SeqCst.
                let val = U128 { whole: val };
                let (mut prev_hi, mut prev_lo);
                asm!(
                    "lpq %r0, 0({dst})",
                    "2:",
                        $($op)*
                        "cdsg %r0, %r12, 0({dst})",
                        "jl 2b",
                    dst = in(reg) ptr_reg!(dst),
                    val_hi = in(reg) val.pair.hi,
                    val_lo = in(reg) val.pair.lo,
                    $($reg)*
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    out("r0") prev_hi,
                    out("r1") prev_lo,
                    out("r12") _,
                    out("r13") _,
                    // Do not use `preserves_flags` because CDSG modifies the condition code.
                    options(nostack),
                );
                U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
            }
        }
    };
}
/// Atomic RMW by CAS loop (2 arguments)
/// `unsafe fn(dst: *mut u128, order: Ordering) -> u128;`
///
/// `$op` can use the following registers:
/// - r0/r1 pair: previous value loaded (read-only for `$op`)
/// - r12/r13 pair: new value that will be stored
// We could use atomic_update here, but using an inline assembly allows omitting
// the comparison of results and the storing/comparing of condition flags.
macro_rules! atomic_rmw_cas_2 {
    ($name:ident, $($op:tt)*) => {
        #[inline]
        unsafe fn $name(dst: *mut u128, _order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                // atomic RMW is always SeqCst.
                let (mut prev_hi, mut prev_lo);
                asm!(
                    "lpq %r0, 0({dst})",
                    "2:",
                        $($op)*
                        "cdsg %r0, %r12, 0({dst})",
                        "jl 2b",
                    dst = in(reg) ptr_reg!(dst),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    out("r0") prev_hi,
                    out("r1") prev_lo,
                    out("r12") _,
                    out("r13") _,
                    // Do not use `preserves_flags` because CDSG modifies the condition code.
                    options(nostack),
                );
                U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
            }
        }
    };
}

atomic_rmw_cas_3! {
    atomic_add, [],
    distinct_op!("algr", "%r13", "%r1", "{val_lo}"),
    "lgr %r12, %r0",
    "alcgr %r12, {val_hi}",
}
atomic_rmw_cas_3! {
    atomic_sub, [],
    distinct_op!("slgr", "%r13", "%r1", "{val_lo}"),
    "lgr %r12, %r0",
    "slbgr %r12, {val_hi}",
}
atomic_rmw_cas_3! {
    atomic_and, [],
    distinct_op!("ngr", "%r13", "%r1", "{val_lo}"),
    distinct_op!("ngr", "%r12", "%r0", "{val_hi}"),
}

// Use nngrk on z15 or later.
#[cfg(any(
    target_feature = "miscellaneous-extensions-3",
    portable_atomic_target_feature = "miscellaneous-extensions-3",
))]
atomic_rmw_cas_3! {
    atomic_nand, [],
    "nngrk %r13, %r1, {val_lo}",
    "nngrk %r12, %r0, {val_hi}",
}
#[cfg(not(any(
    target_feature = "miscellaneous-extensions-3",
    portable_atomic_target_feature = "miscellaneous-extensions-3",
)))]
atomic_rmw_cas_3! {
    atomic_nand, [],
    distinct_op!("ngr", "%r13", "%r1", "{val_lo}"),
    "xihf %r13, 4294967295",
    "xilf %r13, 4294967295",
    distinct_op!("ngr", "%r12", "%r0", "{val_hi}"),
    "xihf %r12, 4294967295",
    "xilf %r12, 4294967295",
}

atomic_rmw_cas_3! {
    atomic_or, [],
    distinct_op!("ogr", "%r13", "%r1", "{val_lo}"),
    distinct_op!("ogr", "%r12", "%r0", "{val_hi}"),
}
atomic_rmw_cas_3! {
    atomic_xor, [],
    distinct_op!("xgr", "%r13", "%r1", "{val_lo}"),
    distinct_op!("xgr", "%r12", "%r0", "{val_hi}"),
}

#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
atomic_rmw_cas_3! {
    atomic_max, [],
    "clgr %r1, {val_lo}",
    select_op!("h", "%r12", "%r1", "{val_lo}"),
    "cgr %r0, {val_hi}",
    select_op!("h", "%r13", "%r1", "{val_lo}"),
    "locgre %r13, %r12",
    select_op!("h", "%r12", "%r0", "{val_hi}"),
}
#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
atomic_rmw_cas_3! {
    atomic_umax, [tmp = out(reg) _,],
    "clgr %r1, {val_lo}",
    select_op!("h", "{tmp}", "%r1", "{val_lo}"),
    "clgr %r0, {val_hi}",
    select_op!("h", "%r12", "%r0", "{val_hi}"),
    select_op!("h", "%r13", "%r1", "{val_lo}"),
    "cgr %r0, {val_hi}",
    "locgre %r13, {tmp}",
}
#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
atomic_rmw_cas_3! {
    atomic_min, [],
    "clgr %r1, {val_lo}",
    select_op!("l", "%r12", "%r1", "{val_lo}"),
    "cgr %r0, {val_hi}",
    select_op!("l", "%r13", "%r1", "{val_lo}"),
    "locgre %r13, %r12",
    select_op!("l", "%r12", "%r0", "{val_hi}"),
}
#[cfg(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
))]
atomic_rmw_cas_3! {
    atomic_umin, [tmp = out(reg) _,],
    "clgr %r1, {val_lo}",
    select_op!("l", "{tmp}", "%r1", "{val_lo}"),
    "clgr %r0, {val_hi}",
    select_op!("l", "%r12", "%r0", "{val_hi}"),
    select_op!("l", "%r13", "%r1", "{val_lo}"),
    "cgr %r0, {val_hi}",
    "locgre %r13, {tmp}",
}
// We use atomic_update for atomic min/max on pre-z196 because
// z10 doesn't seem to have a good way to implement 128-bit min/max.
// loc{,g}r requires z196 or later.
// https://godbolt.org/z/j8KG9q5oq
#[cfg(not(any(
    target_feature = "load-store-on-cond",
    portable_atomic_target_feature = "load-store-on-cond",
)))]
atomic_rmw_by_atomic_update!(cmp);

atomic_rmw_cas_2! {
    atomic_not,
    "lgr %r13, %r1",
    "xihf %r13, 4294967295",
    "xilf %r13, 4294967295",
    "lgr %r12, %r0",
    "xihf %r12, 4294967295",
    "xilf %r12, 4294967295",
}
atomic_rmw_cas_2! {
    atomic_neg,
    "lghi %r13, 0",
    "slgr %r13, %r1",
    "lghi %r12, 0",
    "slbgr %r12, %r0",
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
