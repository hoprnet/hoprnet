// Refs: https://developer.arm.com/documentation/ddi0406/cb/System-Level-Architecture/The-System-Level-Programmers--Model/ARM-processor-modes-and-ARM-core-registers/Program-Status-Registers--PSRs-?lang=en
//
// Generated asm:
// - armv5te https://godbolt.org/z/5arYrfzYc

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;

#[cfg(not(portable_atomic_disable_fiq))]
macro_rules! if_disable_fiq {
    ($tt:tt) => {
        ""
    };
}
#[cfg(portable_atomic_disable_fiq)]
macro_rules! if_disable_fiq {
    ($tt:tt) => {
        $tt
    };
}

pub(super) type State = u32;

/// Disables interrupts and returns the previous interrupt state.
#[inline]
#[instruction_set(arm::a32)]
pub(super) fn disable() -> State {
    let cpsr: State;
    // SAFETY: reading CPSR and disabling interrupts are safe.
    // (see module-level comments of interrupt/mod.rs on the safety of using privileged instructions)
    unsafe {
        // Do not use `nomem` and `readonly` because prevent subsequent memory accesses from being reordered before interrupts are disabled.
        asm!(
            "mrs {prev}, cpsr",
            "orr {new}, {prev}, 0x80", // I (IRQ mask) bit (1 << 7)
            // We disable only IRQs by default. See also https://github.com/taiki-e/portable-atomic/pull/28#issuecomment-1214146912.
            if_disable_fiq!("orr {new}, {new}, 0x40"), // F (FIQ mask) bit (1 << 6)
            "msr cpsr_c, {new}",
            prev = out(reg) cpsr,
            new = out(reg) _,
            options(nostack, preserves_flags),
        );
    }
    cpsr
}

/// Restores the previous interrupt state.
///
/// # Safety
///
/// The state must be the one retrieved by the previous `disable`.
#[inline]
#[instruction_set(arm::a32)]
pub(super) unsafe fn restore(cpsr: State) {
    // SAFETY: the caller must guarantee that the state was retrieved by the previous `disable`,
    unsafe {
        // This clobbers the entire CPSR. See msp430.rs to safety on this.
        //
        // Do not use `nomem` and `readonly` because prevent preceding memory accesses from being reordered after interrupts are enabled.
        asm!("msr cpsr_c, {0}", in(reg) cpsr, options(nostack));
    }
}

// On pre-v6 ARM, we cannot use core::sync::atomic here because they call the
// `__sync_*` builtins for non-relaxed load/store (because pre-v6 ARM doesn't
// have Data Memory Barrier).
//
// Generated asm:
// - armv5te https://godbolt.org/z/a7zcs9hKa
pub(crate) mod atomic {
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

            impl $(<$($generics)*>)? $atomic_type $(<$($generics)*>)? {
                #[inline]
                pub(crate) fn load(&self, order: Ordering) -> $value_type {
                    let src = self.v.get();
                    // SAFETY: any data races are prevented by atomic intrinsics and the raw
                    // pointer passed in is valid because we got it from a reference.
                    unsafe {
                        let out;
                        match order {
                            Ordering::Relaxed => {
                                asm!(
                                    concat!("ldr", $asm_suffix, " {out}, [{src}]"),
                                    src = in(reg) src,
                                    out = lateout(reg) out,
                                    options(nostack, preserves_flags, readonly),
                                );
                            }
                            Ordering::Acquire | Ordering::SeqCst => {
                                // inline asm without nomem/readonly implies compiler fence.
                                // And compiler fence is fine because the user explicitly declares that
                                // the system is single-core by using an unsafe cfg.
                                asm!(
                                    concat!("ldr", $asm_suffix, " {out}, [{src}]"),
                                    src = in(reg) src,
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
                pub(crate) fn store(&self, val: $value_type, _order: Ordering) {
                    let dst = self.v.get();
                    // SAFETY: any data races are prevented by atomic intrinsics and the raw
                    // pointer passed in is valid because we got it from a reference.
                    unsafe {
                        // inline asm without nomem/readonly implies compiler fence.
                        // And compiler fence is fine because the user explicitly declares that
                        // the system is single-core by using an unsafe cfg.
                        asm!(
                            concat!("str", $asm_suffix, " {val}, [{dst}]"),
                            dst = in(reg) dst,
                            val = in(reg) val,
                            options(nostack, preserves_flags),
                        );
                    }
                }
            }
        };
    }

    atomic!(AtomicI8, i8, "b");
    atomic!(AtomicU8, u8, "b");
    atomic!(AtomicI16, i16, "h");
    atomic!(AtomicU16, u16, "h");
    atomic!(AtomicI32, i32, "");
    atomic!(AtomicU32, u32, "");
    atomic!(AtomicIsize, isize, "");
    atomic!(AtomicUsize, usize, "");
    atomic!([T] AtomicPtr, *mut T, "");
}
