// SPDX-License-Identifier: Apache-2.0 OR MIT

// Refs:
// - Xtensa Instruction Set Architecture (ISA) Reference Manual https://0x04.net/~mwk/doc/xtensa.pdf
// - Linux kernel's Xtensa atomic implementation https://github.com/torvalds/linux/blob/v6.1/arch/xtensa/include/asm/atomic.h

use core::arch::asm;

pub(super) use core::sync::atomic;

pub(super) type State = u32;

/// Disables interrupts and returns the previous interrupt state.
#[inline]
pub(super) fn disable() -> State {
    let r: State;
    // SAFETY: reading the PS special register and disabling all interrupts is safe.
    // (see module-level comments of interrupt/mod.rs on the safety of using privileged instructions)
    unsafe {
        // Do not use `nomem` and `readonly` because prevent subsequent memory accesses from being reordered before interrupts are disabled.
        // Interrupt level 15 to disable all interrupts.
        // SYNC after RSIL is not required.
        asm!("rsil {0}, 15", out(reg) r, options(nostack));
    }
    r
}

/// Restores the previous interrupt state.
///
/// # Safety
///
/// The state must be the one retrieved by the previous `disable`.
#[inline]
pub(super) unsafe fn restore(r: State) {
    // SAFETY: the caller must guarantee that the state was retrieved by the previous `disable`,
    // and we've checked that interrupts were enabled before disabling interrupts.
    unsafe {
        // Do not use `nomem` and `readonly` because prevent preceding memory accesses from being reordered after interrupts are enabled.
        // SYNC after WSR is required to guarantee that subsequent RSIL read the written value.
        asm!(
            "wsr.ps {0}",
            "rsync",
            in(reg) r,
            options(nostack),
        );
    }
}
