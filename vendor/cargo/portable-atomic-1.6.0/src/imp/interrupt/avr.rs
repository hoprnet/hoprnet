// SPDX-License-Identifier: Apache-2.0 OR MIT

// Adapted from https://github.com/Rahix/avr-device.
//
// Refs:
// - AVR Instruction Set Manual https://ww1.microchip.com/downloads/en/DeviceDoc/AVR-InstructionSet-Manual-DS40002198.pdf

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;

pub(super) type State = u8;

/// Disables interrupts and returns the previous interrupt state.
#[inline]
pub(super) fn disable() -> State {
    let sreg: State;
    // SAFETY: reading the status register (SREG) and disabling interrupts are safe.
    // (see module-level comments of interrupt/mod.rs on the safety of using privileged instructions)
    unsafe {
        // Do not use `nomem` and `readonly` because prevent subsequent memory accesses from being reordered before interrupts are disabled.
        // Do not use `preserves_flags` because CLI modifies the I bit of the status register (SREG).
        // Refs: https://ww1.microchip.com/downloads/en/DeviceDoc/AVR-InstructionSet-Manual-DS40002198.pdf#page=58
        #[cfg(not(portable_atomic_no_asm))]
        asm!(
            "in {0}, 0x3F",
            "cli",
            out(reg) sreg,
            options(nostack),
        );
        #[cfg(portable_atomic_no_asm)]
        {
            llvm_asm!("in $0, 0x3F" : "=r"(sreg) ::: "volatile");
            llvm_asm!("cli" ::: "memory" : "volatile");
        }
    }
    sreg
}

/// Restores the previous interrupt state.
///
/// # Safety
///
/// The state must be the one retrieved by the previous `disable`.
#[inline]
pub(super) unsafe fn restore(sreg: State) {
    // SAFETY: the caller must guarantee that the state was retrieved by the previous `disable`,
    unsafe {
        // This clobbers the entire status register. See msp430.rs to safety on this.
        //
        // Do not use `nomem` and `readonly` because prevent preceding memory accesses from being reordered after interrupts are enabled.
        // Do not use `preserves_flags` because OUT modifies the status register (SREG).
        #[cfg(not(portable_atomic_no_asm))]
        asm!("out 0x3F, {0}", in(reg) sreg, options(nostack));
        #[cfg(portable_atomic_no_asm)]
        llvm_asm!("out 0x3F, $0" :: "r"(sreg) : "memory" : "volatile");
    }
}
