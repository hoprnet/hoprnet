// SPDX-License-Identifier: Apache-2.0 OR MIT

// Atomic{I,U}128 implementation on PowerPC64.
//
// powerpc64 on pwr8+ support 128-bit atomics:
// https://github.com/llvm/llvm-project/commit/549e118e93c666914a1045fde38a2cac33e1e445
// https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/test/CodeGen/PowerPC/atomics-i128-ldst.ll
// https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/test/CodeGen/PowerPC/atomics-i128.ll
//
// powerpc64le is pwr8+ by default https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/lib/Target/PowerPC/PPC.td#L663
// See also https://github.com/rust-lang/rust/issues/59932
//
// Note that we do not separate LL and SC into separate functions, but handle
// them within a single asm block. This is because it is theoretically possible
// for the compiler to insert operations that might clear the reservation between
// LL and SC. See aarch64.rs for details.
//
// Note: On Miri and ThreadSanitizer which do not support inline assembly, we don't use
// this module and use intrinsics.rs instead.
//
// Refs:
// - Power ISA https://openpowerfoundation.org/specifications/isa
// - AIX Assembler language reference https://www.ibm.com/docs/en/aix/7.3?topic=aix-assembler-language-reference
// - atomic-maybe-uninit https://github.com/taiki-e/atomic-maybe-uninit
//
// Generated asm:
// - powerpc64 (pwr8) https://godbolt.org/z/nG5dGa38a
// - powerpc64le https://godbolt.org/z/6c99s75e4

include!("macros.rs");

#[cfg(not(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
)))]
#[path = "../fallback/outline_atomics.rs"]
mod fallback;

// On musl with static linking, it seems that getauxval is not always available.
// See detect/auxv.rs for more.
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(test, portable_atomic_outline_atomics))] // TODO(powerpc64): currently disabled by default
#[cfg(any(
    test,
    not(any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
    )),
))]
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

use core::{arch::asm, sync::atomic::Ordering};

use crate::utils::{Pair, U128};

macro_rules! debug_assert_pwr8 {
    () => {
        #[cfg(not(any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        )))]
        {
            debug_assert!(detect::detect().has_quadword_atomics());
        }
    };
}

// Refs: https://www.ibm.com/docs/en/aix/7.3?topic=ops-machine-pseudo-op
//
// This is similar to #[target_feature(enable = "quadword-atomics")], except that there are
// no compiler guarantees regarding (un)inlining, and the scope is within an asm
// block rather than a function. We use this directive because #[target_feature(enable = "quadword-atomics")]
// is not supported as of Rust 1.70-nightly.
//
// start_pwr8 and end_pwr8 must be used in pairs.
//
// Note: If power8 instructions are not available at compile-time, we must guarantee that
// the function that uses it is not inlined into a function where it is not
// clear whether power8 instructions are available. Otherwise, (even if we checked whether
// power8 instructions are available at run-time) optimizations that reorder its
// instructions across the if condition might introduce undefined behavior.
// (see also https://rust-lang.github.io/rfcs/2045-target-feature.html#safely-inlining-target_feature-functions-on-more-contexts)
// However, our code uses the ifunc helper macro that works with function pointers,
// so we don't have to worry about this unless calling without helper macro.
macro_rules! start_pwr8 {
    () => {
        ".machine push\n.machine power8"
    };
}
macro_rules! end_pwr8 {
    () => {
        ".machine pop"
    };
}

macro_rules! atomic_rmw {
    ($op:ident, $order:ident) => {
        match $order {
            Ordering::Relaxed => $op!("", ""),
            Ordering::Acquire => $op!("lwsync", ""),
            Ordering::Release => $op!("", "lwsync"),
            Ordering::AcqRel => $op!("lwsync", "lwsync"),
            Ordering::SeqCst => $op!("lwsync", "sync"),
            _ => unreachable!("{:?}", $order),
        }
    };
}

// Extracts and checks the EQ bit of cr0.
#[inline]
fn extract_cr0(r: u64) -> bool {
    r & 0x20000000 != 0
}

#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
use atomic_load_pwr8 as atomic_load;
#[cfg(not(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
)))]
#[inline]
unsafe fn atomic_load(src: *mut u128, order: Ordering) -> u128 {
    fn_alias! {
        // inline(never) is just a hint and also not strictly necessary
        // because we use ifunc helper macro, but used for clarity.
        #[inline(never)]
        unsafe fn(src: *mut u128) -> u128;
        atomic_load_pwr8_relaxed = atomic_load_pwr8(Ordering::Relaxed);
        atomic_load_pwr8_acquire = atomic_load_pwr8(Ordering::Acquire);
        atomic_load_pwr8_seqcst = atomic_load_pwr8(Ordering::SeqCst);
    }
    // SAFETY: the caller must uphold the safety contract.
    // we only calls atomic_load_pwr8 if quadword-atomics is available.
    unsafe {
        match order {
            Ordering::Relaxed => {
                ifunc!(unsafe fn(src: *mut u128) -> u128 {
                    if detect::detect().has_quadword_atomics() {
                        atomic_load_pwr8_relaxed
                    } else {
                        fallback::atomic_load_non_seqcst
                    }
                })
            }
            Ordering::Acquire => {
                ifunc!(unsafe fn(src: *mut u128) -> u128 {
                    if detect::detect().has_quadword_atomics() {
                        atomic_load_pwr8_acquire
                    } else {
                        fallback::atomic_load_non_seqcst
                    }
                })
            }
            Ordering::SeqCst => {
                ifunc!(unsafe fn(src: *mut u128) -> u128 {
                    if detect::detect().has_quadword_atomics() {
                        atomic_load_pwr8_seqcst
                    } else {
                        fallback::atomic_load_seqcst
                    }
                })
            }
            _ => unreachable!("{:?}", order),
        }
    }
}
#[inline]
unsafe fn atomic_load_pwr8(src: *mut u128, order: Ordering) -> u128 {
    debug_assert!(src as usize % 16 == 0);
    debug_assert_pwr8!();

    // SAFETY: the caller must uphold the safety contract.
    //
    // Refs: "3.3.4 Fixed Point Load and Store Quadword Instructions" of Power ISA
    unsafe {
        let (out_hi, out_lo);
        macro_rules! atomic_load_acquire {
            ($release:tt) => {
                asm!(
                    start_pwr8!(),
                    $release,
                    "lq %r4, 0({src})",
                    // Lightweight acquire sync
                    // Refs: https://github.com/boostorg/atomic/blob/boost-1.79.0/include/boost/atomic/detail/core_arch_ops_gcc_ppc.hpp#L47-L62
                    "cmpd %cr7, %r4, %r4",
                    "bne- %cr7, 2f",
                    "2:",
                    "isync",
                    end_pwr8!(),
                    src = in(reg_nonzero) ptr_reg!(src),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    out("r4") out_hi,
                    out("r5") out_lo,
                    out("cr7") _,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            Ordering::Relaxed => {
                asm!(
                    start_pwr8!(),
                    "lq %r4, 0({src})",
                    end_pwr8!(),
                    src = in(reg_nonzero) ptr_reg!(src),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    out("r4") out_hi,
                    out("r5") out_lo,
                    options(nostack, preserves_flags, readonly),
                );
            }
            Ordering::Acquire => atomic_load_acquire!(""),
            Ordering::SeqCst => atomic_load_acquire!("sync"),
            _ => unreachable!("{:?}", order),
        }
        U128 { pair: Pair { hi: out_hi, lo: out_lo } }.whole
    }
}

#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
use atomic_store_pwr8 as atomic_store;
#[cfg(not(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
)))]
#[inline]
unsafe fn atomic_store(dst: *mut u128, val: u128, order: Ordering) {
    fn_alias! {
        // inline(never) is just a hint and also not strictly necessary
        // because we use ifunc helper macro, but used for clarity.
        #[inline(never)]
        unsafe fn(dst: *mut u128, val: u128);
        atomic_store_pwr8_relaxed = atomic_store_pwr8(Ordering::Relaxed);
        atomic_store_pwr8_release = atomic_store_pwr8(Ordering::Release);
        atomic_store_pwr8_seqcst = atomic_store_pwr8(Ordering::SeqCst);
    }
    // SAFETY: the caller must uphold the safety contract.
    // we only calls atomic_store_pwr8 if quadword-atomics is available.
    unsafe {
        match order {
            Ordering::Relaxed => {
                ifunc!(unsafe fn(dst: *mut u128, val: u128) {
                    if detect::detect().has_quadword_atomics() {
                        atomic_store_pwr8_relaxed
                    } else {
                        fallback::atomic_store_non_seqcst
                    }
                });
            }
            Ordering::Release => {
                ifunc!(unsafe fn(dst: *mut u128, val: u128) {
                    if detect::detect().has_quadword_atomics() {
                        atomic_store_pwr8_release
                    } else {
                        fallback::atomic_store_non_seqcst
                    }
                });
            }
            Ordering::SeqCst => {
                ifunc!(unsafe fn(dst: *mut u128, val: u128) {
                    if detect::detect().has_quadword_atomics() {
                        atomic_store_pwr8_seqcst
                    } else {
                        fallback::atomic_store_seqcst
                    }
                });
            }
            _ => unreachable!("{:?}", order),
        }
    }
}
#[inline]
unsafe fn atomic_store_pwr8(dst: *mut u128, val: u128, order: Ordering) {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_pwr8!();

    // SAFETY: the caller must uphold the safety contract.
    //
    // Refs: "3.3.4 Fixed Point Load and Store Quadword Instructions" of Power ISA
    unsafe {
        let val = U128 { whole: val };
        macro_rules! atomic_store {
            ($release:tt) => {
                asm!(
                    start_pwr8!(),
                    $release,
                    "stq %r4, 0({dst})",
                    end_pwr8!(),
                    dst = in(reg_nonzero) ptr_reg!(dst),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    in("r4") val.pair.hi,
                    in("r5") val.pair.lo,
                    options(nostack, preserves_flags),
                )
            };
        }
        match order {
            Ordering::Relaxed => atomic_store!(""),
            Ordering::Release => atomic_store!("lwsync"),
            Ordering::SeqCst => atomic_store!("sync"),
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
    let success = crate::utils::upgrade_success_ordering(success, failure);

    #[cfg(any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
    ))]
    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantees that quadword atomics instructions are available at compile-time.
    let (prev, ok) = unsafe { atomic_compare_exchange_pwr8(dst, old, new, success) };
    #[cfg(not(any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
    )))]
    // SAFETY: the caller must uphold the safety contract.
    let (prev, ok) = unsafe { atomic_compare_exchange_ifunc(dst, old, new, success) };
    if ok {
        Ok(prev)
    } else {
        Err(prev)
    }
}
#[inline]
unsafe fn atomic_compare_exchange_pwr8(
    dst: *mut u128,
    old: u128,
    new: u128,
    order: Ordering,
) -> (u128, bool) {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_pwr8!();

    // SAFETY: the caller must uphold the safety contract.
    //
    // Refs: "4.6.2.2 128-bit Load And Reserve and Store Conditional Instructions" of Power ISA
    unsafe {
        let old = U128 { whole: old };
        let new = U128 { whole: new };
        let (mut prev_hi, mut prev_lo);
        let mut r;
        macro_rules! cmpxchg {
            ($acquire:tt, $release:tt) => {
                asm!(
                    start_pwr8!(),
                    $release,
                    "2:",
                        "lqarx %r8, 0, {dst}",
                        "xor {tmp_lo}, %r9, {old_lo}",
                        "xor {tmp_hi}, %r8, {old_hi}",
                        "or. {tmp_lo}, {tmp_lo}, {tmp_hi}",
                        "bne %cr0, 3f", // jump if compare failed
                        "stqcx. %r6, 0, {dst}",
                        "bne %cr0, 2b", // continue loop if store failed
                    "3:",
                    // if compare failed EQ bit is cleared, if stqcx succeeds EQ bit is set.
                    "mfcr {tmp_lo}",
                    $acquire,
                    end_pwr8!(),
                    dst = in(reg_nonzero) ptr_reg!(dst),
                    old_hi = in(reg) old.pair.hi,
                    old_lo = in(reg) old.pair.lo,
                    tmp_hi = out(reg) _,
                    tmp_lo = out(reg) r,
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    in("r6") new.pair.hi,
                    in("r7") new.pair.lo,
                    out("r8") prev_hi,
                    out("r9") prev_lo,
                    out("cr0") _,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw!(cmpxchg, order);
        (U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole, extract_cr0(r))
    }
}

// Always use strong CAS for outline-atomics.
#[cfg(not(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
)))]
use atomic_compare_exchange as atomic_compare_exchange_weak;
#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
#[inline]
unsafe fn atomic_compare_exchange_weak(
    dst: *mut u128,
    old: u128,
    new: u128,
    success: Ordering,
    failure: Ordering,
) -> Result<u128, u128> {
    let success = crate::utils::upgrade_success_ordering(success, failure);

    // SAFETY: the caller must uphold the safety contract.
    // cfg guarantees that quadword atomics instructions are available at compile-time.
    let (prev, ok) = unsafe { atomic_compare_exchange_weak_pwr8(dst, old, new, success) };
    if ok {
        Ok(prev)
    } else {
        Err(prev)
    }
}
#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
#[inline]
unsafe fn atomic_compare_exchange_weak_pwr8(
    dst: *mut u128,
    old: u128,
    new: u128,
    order: Ordering,
) -> (u128, bool) {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_pwr8!();

    // SAFETY: the caller must uphold the safety contract.
    //
    // Refs: "4.6.2.2 128-bit Load And Reserve and Store Conditional Instructions" of Power ISA
    unsafe {
        let old = U128 { whole: old };
        let new = U128 { whole: new };
        let (mut prev_hi, mut prev_lo);
        let mut r;
        macro_rules! cmpxchg_weak {
            ($acquire:tt, $release:tt) => {
                asm!(
                    start_pwr8!(),
                    $release,
                    "lqarx %r8, 0, {dst}",
                    "xor {tmp_lo}, %r9, {old_lo}",
                    "xor {tmp_hi}, %r8, {old_hi}",
                    "or. {tmp_lo}, {tmp_lo}, {tmp_hi}",
                    "bne %cr0, 3f", // jump if compare failed
                    "stqcx. %r6, 0, {dst}",
                    "3:",
                    // if compare or stqcx failed EQ bit is cleared, if stqcx succeeds EQ bit is set.
                    "mfcr {tmp_lo}",
                    $acquire,
                    end_pwr8!(),
                    dst = in(reg_nonzero) ptr_reg!(dst),
                    old_hi = in(reg) old.pair.hi,
                    old_lo = in(reg) old.pair.lo,
                    tmp_hi = out(reg) _,
                    tmp_lo = out(reg) r,
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    in("r6") new.pair.hi,
                    in("r7") new.pair.lo,
                    out("r8") prev_hi,
                    out("r9") prev_lo,
                    out("cr0") _,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw!(cmpxchg_weak, order);
        (U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole, extract_cr0(r))
    }
}

#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
use atomic_swap_pwr8 as atomic_swap;
// Do not use atomic_rmw_ll_sc_3 because it needs extra MR to implement swap.
#[inline]
unsafe fn atomic_swap_pwr8(dst: *mut u128, val: u128, order: Ordering) -> u128 {
    debug_assert!(dst as usize % 16 == 0);
    debug_assert_pwr8!();

    // SAFETY: the caller must uphold the safety contract.
    unsafe {
        let val = U128 { whole: val };
        let (mut prev_hi, mut prev_lo);
        macro_rules! swap {
            ($acquire:tt, $release:tt) => {
                asm!(
                    start_pwr8!(),
                    $release,
                    "2:",
                        "lqarx %r6, 0, {dst}",
                        "stqcx. %r8, 0, {dst}",
                        "bne %cr0, 2b",
                    $acquire,
                    end_pwr8!(),
                    dst = in(reg_nonzero) ptr_reg!(dst),
                    // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                    // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                    out("r6") prev_hi,
                    out("r7") prev_lo,
                    in("r8") val.pair.hi,
                    in("r9") val.pair.lo,
                    out("cr0") _,
                    options(nostack, preserves_flags),
                )
            };
        }
        atomic_rmw!(swap, order);
        U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
    }
}

/// Atomic RMW by LL/SC loop (3 arguments)
/// `unsafe fn(dst: *mut u128, val: u128, order: Ordering) -> u128;`
///
/// $op can use the following registers:
/// - val_hi/val_lo pair: val argument (read-only for `$op`)
/// - r6/r7 pair: previous value loaded by ll (read-only for `$op`)
/// - r8/r9 pair: new value that will be stored by sc
macro_rules! atomic_rmw_ll_sc_3 {
    ($name:ident as $reexport_name:ident, [$($reg:tt)*], $($op:tt)*) => {
        #[cfg(any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        ))]
        use $name as $reexport_name;
        #[inline]
        unsafe fn $name(dst: *mut u128, val: u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            debug_assert_pwr8!();
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                let val = U128 { whole: val };
                let (mut prev_hi, mut prev_lo);
                macro_rules! op {
                    ($acquire:tt, $release:tt) => {
                        asm!(
                            start_pwr8!(),
                            $release,
                            "2:",
                                "lqarx %r6, 0, {dst}",
                                $($op)*
                                "stqcx. %r8, 0, {dst}",
                                "bne %cr0, 2b",
                            $acquire,
                            end_pwr8!(),
                            dst = in(reg_nonzero) ptr_reg!(dst),
                            val_hi = in(reg) val.pair.hi,
                            val_lo = in(reg) val.pair.lo,
                            $($reg)*
                            // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                            // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                            out("r6") prev_hi,
                            out("r7") prev_lo,
                            out("r8") _, // new (hi)
                            out("r9") _, // new (lo)
                            out("cr0") _,
                            options(nostack, preserves_flags),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
            }
        }
    };
}
/// Atomic RMW by LL/SC loop (2 arguments)
/// `unsafe fn(dst: *mut u128, order: Ordering) -> u128;`
///
/// $op can use the following registers:
/// - r6/r7 pair: previous value loaded by ll (read-only for `$op`)
/// - r8/r9 pair: new value that will be stored by sc
macro_rules! atomic_rmw_ll_sc_2 {
    ($name:ident as $reexport_name:ident, [$($reg:tt)*], $($op:tt)*) => {
        #[cfg(any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        ))]
        use $name as $reexport_name;
        #[inline]
        unsafe fn $name(dst: *mut u128, order: Ordering) -> u128 {
            debug_assert!(dst as usize % 16 == 0);
            debug_assert_pwr8!();
            // SAFETY: the caller must uphold the safety contract.
            unsafe {
                let (mut prev_hi, mut prev_lo);
                macro_rules! op {
                    ($acquire:tt, $release:tt) => {
                        asm!(
                            start_pwr8!(),
                            $release,
                            "2:",
                                "lqarx %r6, 0, {dst}",
                                $($op)*
                                "stqcx. %r8, 0, {dst}",
                                "bne %cr0, 2b",
                            $acquire,
                            end_pwr8!(),
                            dst = in(reg_nonzero) ptr_reg!(dst),
                            $($reg)*
                            // Quadword atomic instructions work with even/odd pair of specified register and subsequent register.
                            // We cannot use r1 (sp) and r2 (system reserved), so start with r4 or grater.
                            out("r6") prev_hi,
                            out("r7") prev_lo,
                            out("r8") _, // new (hi)
                            out("r9") _, // new (lo)
                            out("cr0") _,
                            options(nostack, preserves_flags),
                        )
                    };
                }
                atomic_rmw!(op, order);
                U128 { pair: Pair { hi: prev_hi, lo: prev_lo } }.whole
            }
        }
    };
}

atomic_rmw_ll_sc_3! {
    atomic_add_pwr8 as atomic_add, [out("xer") _,],
    "addc %r9, {val_lo}, %r7",
    "adde %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_sub_pwr8 as atomic_sub, [out("xer") _,],
    "subc %r9, %r7, {val_lo}",
    "subfe %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_and_pwr8 as atomic_and, [],
    "and %r9, {val_lo}, %r7",
    "and %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_nand_pwr8 as atomic_nand, [],
    "nand %r9, {val_lo}, %r7",
    "nand %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_or_pwr8 as atomic_or, [],
    "or %r9, {val_lo}, %r7",
    "or %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_xor_pwr8 as atomic_xor, [],
    "xor %r9, {val_lo}, %r7",
    "xor %r8, {val_hi}, %r6",
}
atomic_rmw_ll_sc_3! {
    atomic_max_pwr8 as atomic_max, [out("cr1") _,],
    "cmpld %r7, {val_lo}",        // (unsigned) compare lo 64-bit, store result to cr0
    "iselgt %r9, %r7, {val_lo}",  // select lo 64-bit based on GT bit in cr0
    "cmpd %cr1, %r6, {val_hi}",   // (signed) compare hi 64-bit, store result to cr1
    "isel %r8, %r7, {val_lo}, 5", // select lo 64-bit based on GT bit in cr1
    "cmpld %r6, {val_hi}",        // (unsigned) compare hi 64-bit, store result to cr0
    "iseleq %r9, %r9, %r8",       // select lo 64-bit based on EQ bit in cr0
    "isel %r8, %r6, {val_hi}, 5", // select hi 64-bit based on GT bit in cr1
}
atomic_rmw_ll_sc_3! {
    atomic_umax_pwr8 as atomic_umax, [],
    "cmpld %r7, {val_lo}",       // compare lo 64-bit, store result to cr0
    "iselgt %r9, %r7, {val_lo}", // select lo 64-bit based on GT bit in cr0
    "cmpld %r6, {val_hi}",       // compare hi 64-bit, store result to cr0
    "iselgt %r8, %r7, {val_lo}", // select lo 64-bit based on GT bit in cr0
    "iseleq %r9, %r9, %r8",      // select lo 64-bit based on EQ bit in cr0
    "iselgt %r8, %r6, {val_hi}", // select hi 64-bit based on GT bit in cr0
}
atomic_rmw_ll_sc_3! {
    atomic_min_pwr8 as atomic_min, [out("cr1") _,],
    "cmpld %r7, {val_lo}",        // (unsigned) compare lo 64-bit, store result to cr0
    "isellt %r9, %r7, {val_lo}",  // select lo 64-bit based on LT bit in cr0
    "cmpd %cr1, %r6, {val_hi}",   // (signed) compare hi 64-bit, store result to cr1
    "isel %r8, %r7, {val_lo}, 4", // select lo 64-bit based on LT bit in cr1
    "cmpld %r6, {val_hi}",        // (unsigned) compare hi 64-bit, store result to cr0
    "iseleq %r9, %r9, %r8",       // select lo 64-bit based on EQ bit in cr0
    "isel %r8, %r6, {val_hi}, 4", // select hi 64-bit based on LT bit in cr1
}
atomic_rmw_ll_sc_3! {
    atomic_umin_pwr8 as atomic_umin, [],
    "cmpld %r7, {val_lo}",       // compare lo 64-bit, store result to cr0
    "isellt %r9, %r7, {val_lo}", // select lo 64-bit based on LT bit in cr0
    "cmpld %r6, {val_hi}",       // compare hi 64-bit, store result to cr0
    "isellt %r8, %r7, {val_lo}", // select lo 64-bit based on LT bit in cr0
    "iseleq %r9, %r9, %r8",      // select lo 64-bit based on EQ bit in cr0
    "isellt %r8, %r6, {val_hi}", // select hi 64-bit based on LT bit in cr0
}

#[cfg(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
))]
use atomic_not_pwr8 as atomic_not;
#[inline]
unsafe fn atomic_not_pwr8(dst: *mut u128, order: Ordering) -> u128 {
    // SAFETY: the caller must uphold the safety contract.
    unsafe { atomic_xor_pwr8(dst, !0, order) }
}

#[cfg(portable_atomic_llvm_16)]
atomic_rmw_ll_sc_2! {
    atomic_neg_pwr8 as atomic_neg, [out("xer") _,],
    "subfic %r9, %r7, 0",
    "subfze %r8, %r6",
}
// LLVM 15 miscompiles subfic.
#[cfg(not(portable_atomic_llvm_16))]
atomic_rmw_ll_sc_2! {
    atomic_neg_pwr8 as atomic_neg, [zero = in(reg) 0_u64, out("xer") _,],
    "subc %r9, {zero}, %r7",
    "subfze %r8, %r6",
}

macro_rules! atomic_rmw_with_ifunc {
    (
        unsafe fn $name:ident($($arg:tt)*) $(-> $ret_ty:ty)?;
        pwr8 = $pwr8_fn:ident;
        non_seqcst_fallback = $non_seqcst_fallback_fn:ident;
        seqcst_fallback = $seqcst_fallback_fn:ident;
    ) => {
        #[cfg(not(any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        )))]
        #[inline]
        unsafe fn $name($($arg)*, order: Ordering) $(-> $ret_ty)? {
            fn_alias! {
                // inline(never) is just a hint and also not strictly necessary
                // because we use ifunc helper macro, but used for clarity.
                #[inline(never)]
                unsafe fn($($arg)*) $(-> $ret_ty)?;
                pwr8_relaxed_fn = $pwr8_fn(Ordering::Relaxed);
                pwr8_acquire_fn = $pwr8_fn(Ordering::Acquire);
                pwr8_release_fn = $pwr8_fn(Ordering::Release);
                pwr8_acqrel_fn = $pwr8_fn(Ordering::AcqRel);
                pwr8_seqcst_fn = $pwr8_fn(Ordering::SeqCst);
            }
            // SAFETY: the caller must uphold the safety contract.
            // we only calls pwr8_fn if quadword-atomics is available.
            unsafe {
                match order {
                    Ordering::Relaxed => {
                        ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                            if detect::detect().has_quadword_atomics() {
                                pwr8_relaxed_fn
                            } else {
                                fallback::$non_seqcst_fallback_fn
                            }
                        })
                    }
                    Ordering::Acquire => {
                        ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                            if detect::detect().has_quadword_atomics() {
                                pwr8_acquire_fn
                            } else {
                                fallback::$non_seqcst_fallback_fn
                            }
                        })
                    }
                    Ordering::Release => {
                        ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                            if detect::detect().has_quadword_atomics() {
                                pwr8_release_fn
                            } else {
                                fallback::$non_seqcst_fallback_fn
                            }
                        })
                    }
                    Ordering::AcqRel => {
                        ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                            if detect::detect().has_quadword_atomics() {
                                pwr8_acqrel_fn
                            } else {
                                fallback::$non_seqcst_fallback_fn
                            }
                        })
                    }
                    Ordering::SeqCst => {
                        ifunc!(unsafe fn($($arg)*) $(-> $ret_ty)? {
                            if detect::detect().has_quadword_atomics() {
                                pwr8_seqcst_fn
                            } else {
                                fallback::$seqcst_fallback_fn
                            }
                        })
                    }
                    _ => unreachable!("{:?}", order),
                }
            }
        }
    };
}

atomic_rmw_with_ifunc! {
    unsafe fn atomic_compare_exchange_ifunc(dst: *mut u128, old: u128, new: u128) -> (u128, bool);
    pwr8 = atomic_compare_exchange_pwr8;
    non_seqcst_fallback = atomic_compare_exchange_non_seqcst;
    seqcst_fallback = atomic_compare_exchange_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_swap(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_swap_pwr8;
    non_seqcst_fallback = atomic_swap_non_seqcst;
    seqcst_fallback = atomic_swap_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_add(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_add_pwr8;
    non_seqcst_fallback = atomic_add_non_seqcst;
    seqcst_fallback = atomic_add_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_sub(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_sub_pwr8;
    non_seqcst_fallback = atomic_sub_non_seqcst;
    seqcst_fallback = atomic_sub_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_and(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_and_pwr8;
    non_seqcst_fallback = atomic_and_non_seqcst;
    seqcst_fallback = atomic_and_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_nand(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_nand_pwr8;
    non_seqcst_fallback = atomic_nand_non_seqcst;
    seqcst_fallback = atomic_nand_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_or(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_or_pwr8;
    non_seqcst_fallback = atomic_or_non_seqcst;
    seqcst_fallback = atomic_or_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_xor(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_xor_pwr8;
    non_seqcst_fallback = atomic_xor_non_seqcst;
    seqcst_fallback = atomic_xor_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_max(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_max_pwr8;
    non_seqcst_fallback = atomic_max_non_seqcst;
    seqcst_fallback = atomic_max_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_umax(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_umax_pwr8;
    non_seqcst_fallback = atomic_umax_non_seqcst;
    seqcst_fallback = atomic_umax_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_min(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_min_pwr8;
    non_seqcst_fallback = atomic_min_non_seqcst;
    seqcst_fallback = atomic_min_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_umin(dst: *mut u128, val: u128) -> u128;
    pwr8 = atomic_umin_pwr8;
    non_seqcst_fallback = atomic_umin_non_seqcst;
    seqcst_fallback = atomic_umin_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_not(dst: *mut u128) -> u128;
    pwr8 = atomic_not_pwr8;
    non_seqcst_fallback = atomic_not_non_seqcst;
    seqcst_fallback = atomic_not_seqcst;
}
atomic_rmw_with_ifunc! {
    unsafe fn atomic_neg(dst: *mut u128) -> u128;
    pwr8 = atomic_neg_pwr8;
    non_seqcst_fallback = atomic_neg_non_seqcst;
    seqcst_fallback = atomic_neg_seqcst;
}

#[inline]
fn is_lock_free() -> bool {
    #[cfg(any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
    ))]
    {
        // lqarx and stqcx. instructions are statically available.
        true
    }
    #[cfg(not(any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
    )))]
    {
        detect::detect().has_quadword_atomics()
    }
}
const IS_ALWAYS_LOCK_FREE: bool = cfg!(any(
    target_feature = "quadword-atomics",
    portable_atomic_target_feature = "quadword-atomics",
));

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
