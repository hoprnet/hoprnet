# Implementation of disabling interrupts

This module is used to provide atomic CAS for targets where atomic CAS is not available in the standard library.

- On MSP430 and AVR, they are always single-core, so this module is always used.
- On ARMv6-M (thumbv6m), pre-v6 ARM (e.g., thumbv4t, thumbv5te), RISC-V without A-extension, and Xtensa, they could be multi-core, so this module is used when the `unsafe-assume-single-core` feature is enabled.

The implementation uses privileged instructions to disable interrupts, so it usually doesn't work on unprivileged mode.
Enabling this feature in an environment where privileged instructions are not available, or if the instructions used are not sufficient to disable interrupts in the system, it is also usually considered **unsound**, although the details are system-dependent.

Consider using the [`critical-section` feature](../../../README.md#optional-features-critical-section) for systems that cannot use the `unsafe-assume-single-core` feature.

For some targets, the implementation can be changed by explicitly enabling features.

- On ARMv6-M, this disables interrupts by modifying the PRIMASK register.
- On pre-v6 ARM, this disables interrupts by modifying the I (IRQ mask) bit of the CPSR.
- On pre-v6 ARM with the `disable-fiq` feature, this disables interrupts by modifying the I (IRQ mask) bit and F (FIQ mask) bit of the CPSR.
- On RISC-V (without A-extension), this disables interrupts by modifying the MIE (Machine Interrupt Enable) bit of the `mstatus` register.
- On RISC-V (without A-extension) with the `s-mode` feature, this disables interrupts by modifying the SIE (Supervisor Interrupt Enable) bit of the `sstatus` register.
- On RISC-V (without A-extension) with the `force-amo` feature, this uses AMO instructions for RMWs that have corresponding AMO instructions even if A-extension is disabled. For other RMWs, this disables interrupts as usual.
- On MSP430, this disables interrupts by modifying the GIE (Global Interrupt Enable) bit of the status register (SR).
- On AVR, this disables interrupts by modifying the I (Global Interrupt Enable) bit of the status register (SREG).
- On Xtensa, this disables interrupts by modifying the PS special register.

Some operations don't require disabling interrupts (loads and stores on targets except for AVR, but additionally on MSP430 {8,16}-bit `add,sub,and,or,xor,not`, on RISC-V with the `force-amo` feature 32-bit(RV32)/{32,64}-bit(RV64) `swap,fetch_{add,sub,and,or,xor,not,max,min},add,sub,and,or,xor,not` and {8,16}-bit `fetch_{and,or,xor,not},and,or,xor,not`). However, when the `critical-section` feature is enabled, critical sections are taken for all atomic operations.

Feel free to submit an issue if your target is not supported yet.
