// SPDX-License-Identifier: Apache-2.0 OR MIT

// -----------------------------------------------------------------------------
// Lock-free implementations

#[cfg(not(any(
    all(
        portable_atomic_no_atomic_load_store,
        not(all(target_arch = "bpf", not(feature = "critical-section"))),
    ),
    portable_atomic_unsafe_assume_single_core,
    target_arch = "avr",
    target_arch = "msp430",
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        portable_atomic_no_atomic_cas,
    )))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        not(target_has_atomic = "ptr"),
    )))
)]
mod core_atomic;

// aarch64 128-bit atomics
#[cfg(all(
    target_arch = "aarch64",
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
))]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(
    all(any(miri, portable_atomic_sanitize_thread), portable_atomic_new_atomic_intrinsics),
    path = "atomic128/intrinsics.rs"
)]
#[cfg_attr(
    not(all(any(miri, portable_atomic_sanitize_thread), portable_atomic_new_atomic_intrinsics)),
    path = "atomic128/aarch64.rs"
)]
mod aarch64;

// x86_64 128-bit atomics
#[cfg(all(
    target_arch = "x86_64",
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    any(
        target_feature = "cmpxchg16b",
        portable_atomic_target_feature = "cmpxchg16b",
        all(
            feature = "fallback",
            not(portable_atomic_no_cmpxchg16b_target_feature),
            not(portable_atomic_no_outline_atomics),
            not(any(target_env = "sgx", miri)),
        ),
    ),
))]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(any(miri, portable_atomic_sanitize_thread), path = "atomic128/intrinsics.rs")]
#[cfg_attr(not(any(miri, portable_atomic_sanitize_thread)), path = "atomic128/x86_64.rs")]
mod x86_64;

// powerpc64 128-bit atomics
#[cfg(all(
    target_arch = "powerpc64",
    portable_atomic_unstable_asm_experimental_arch,
    any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
        all(
            feature = "fallback",
            not(portable_atomic_no_outline_atomics),
            any(test, portable_atomic_outline_atomics), // TODO(powerpc64): currently disabled by default
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
            ),
            not(any(miri, portable_atomic_sanitize_thread)),
        ),
    ),
))]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(
    all(any(miri, portable_atomic_sanitize_thread), portable_atomic_llvm_15),
    path = "atomic128/intrinsics.rs"
)]
#[cfg_attr(
    not(all(any(miri, portable_atomic_sanitize_thread), portable_atomic_llvm_15)),
    path = "atomic128/powerpc64.rs"
)]
mod powerpc64;

// s390x 128-bit atomics
#[cfg(all(target_arch = "s390x", portable_atomic_unstable_asm_experimental_arch))]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(any(miri, portable_atomic_sanitize_thread), path = "atomic128/intrinsics.rs")]
#[cfg_attr(not(any(miri, portable_atomic_sanitize_thread)), path = "atomic128/s390x.rs")]
mod s390x;

// pre-v6 ARM Linux 64-bit atomics
#[cfg(feature = "fallback")]
// Miri and Sanitizer do not support inline assembly.
#[cfg(all(
    target_arch = "arm",
    not(any(miri, portable_atomic_sanitize_thread)),
    not(portable_atomic_no_asm),
    any(target_os = "linux", target_os = "android"),
    not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
    not(portable_atomic_no_outline_atomics),
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_64))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "64")))]
mod arm_linux;

// MSP430 atomics
#[cfg(target_arch = "msp430")]
pub(crate) mod msp430;

// atomic load/store for RISC-V without A-extension
#[cfg(any(test, not(feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(any(test, portable_atomic_no_atomic_cas)))]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(test, not(target_has_atomic = "ptr")))
)]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod riscv;

// x86-specific optimizations
// Miri and Sanitizer do not support inline assembly.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(any(miri, portable_atomic_sanitize_thread)),
    not(portable_atomic_no_asm),
))]
mod x86;

// -----------------------------------------------------------------------------
// Lock-based fallback implementations

#[cfg(feature = "fallback")]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(not(portable_atomic_no_atomic_cas)))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(target_has_atomic = "ptr"))]
#[cfg(any(
    test,
    not(any(
        all(
            target_arch = "aarch64",
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
        ),
        all(
            target_arch = "x86_64",
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            any(target_feature = "cmpxchg16b", portable_atomic_target_feature = "cmpxchg16b"),
        ),
        all(
            target_arch = "powerpc64",
            portable_atomic_unstable_asm_experimental_arch,
            any(
                target_feature = "quadword-atomics",
                portable_atomic_target_feature = "quadword-atomics",
            ),
        ),
        all(target_arch = "s390x", portable_atomic_unstable_asm_experimental_arch),
    ))
))]
mod fallback;

// -----------------------------------------------------------------------------
// Critical section based fallback implementations

// On AVR, we always use critical section based fallback implementation.
// AVR can be safely assumed to be single-core, so this is sound.
// https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/lib/Target/AVR/AVRExpandPseudoInsts.cpp#L1074
// MSP430 as well.
#[cfg(any(
    all(test, target_os = "none"),
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(any(test, portable_atomic_no_atomic_cas)))]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(test, not(target_has_atomic = "ptr")))
)]
#[cfg(any(
    target_arch = "arm",
    target_arch = "avr",
    target_arch = "msp430",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "xtensa",
    feature = "critical-section",
))]
mod interrupt;

// -----------------------------------------------------------------------------
// Atomic float implementations

#[cfg(feature = "float")]
pub(crate) mod float;

// -----------------------------------------------------------------------------

#[cfg(not(any(
    portable_atomic_no_atomic_load_store,
    portable_atomic_unsafe_assume_single_core,
    target_arch = "avr",
    target_arch = "msp430",
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        portable_atomic_no_atomic_cas,
    )))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        not(target_has_atomic = "ptr"),
    )))
)]
items! {
    pub(crate) use self::core_atomic::{
        AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU32, AtomicU8,
        AtomicUsize,
    };
    #[cfg_attr(
        portable_atomic_no_cfg_target_has_atomic,
        cfg(any(
            not(portable_atomic_no_atomic_64),
            not(any(target_pointer_width = "16", target_pointer_width = "32")),
        ))
    )]
    #[cfg_attr(
        not(portable_atomic_no_cfg_target_has_atomic),
        cfg(any(
            target_has_atomic = "64",
            not(any(target_pointer_width = "16", target_pointer_width = "32")),
        ))
    )]
    pub(crate) use self::core_atomic::{AtomicI64, AtomicU64};
}
// bpf
#[cfg(all(
    target_arch = "bpf",
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
))]
pub(crate) use self::core_atomic::{AtomicI64, AtomicIsize, AtomicPtr, AtomicU64, AtomicUsize};

// RISC-V without A-extension & !(assume single core | critical section)
#[cfg(not(any(portable_atomic_unsafe_assume_single_core, feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
items! {
    pub(crate) use self::riscv::{
        AtomicI16, AtomicI32, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU32, AtomicU8,
        AtomicUsize,
    };
    #[cfg(target_arch = "riscv64")]
    pub(crate) use self::riscv::{AtomicI64, AtomicU64};
}

// no core atomic CAS & (assume single core | critical section) => critical section based fallback
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
items! {
    pub(crate) use self::interrupt::{
        AtomicI16, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU8, AtomicUsize,
    };
    #[cfg(any(not(target_pointer_width = "16"), feature = "fallback"))]
    pub(crate) use self::interrupt::{AtomicI32, AtomicU32};
    #[cfg(any(
        not(any(target_pointer_width = "16", target_pointer_width = "32")),
        feature = "fallback",
    ))]
    pub(crate) use self::interrupt::{AtomicI64, AtomicU64};
    #[cfg(feature = "fallback")]
    pub(crate) use self::interrupt::{AtomicI128, AtomicU128};
}

// no core (64-bit | 128-bit) atomic & has CAS => use lock-base fallback
#[cfg(feature = "fallback")]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(not(portable_atomic_no_atomic_cas)))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(target_has_atomic = "ptr"))]
items! {
    #[cfg(not(all(
        target_arch = "arm",
        not(any(miri, portable_atomic_sanitize_thread)),
        not(portable_atomic_no_asm),
        any(target_os = "linux", target_os = "android"),
        not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
        not(portable_atomic_no_outline_atomics),
    )))]
    #[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_64))]
    #[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "64")))]
    pub(crate) use self::fallback::{AtomicI64, AtomicU64};
    #[cfg(not(any(
        all(
            target_arch = "aarch64",
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
        ),
        all(
            target_arch = "x86_64",
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            any(
                target_feature = "cmpxchg16b",
                portable_atomic_target_feature = "cmpxchg16b",
                all(
                    feature = "fallback",
                    not(portable_atomic_no_cmpxchg16b_target_feature),
                    not(portable_atomic_no_outline_atomics),
                    not(any(target_env = "sgx", miri)),
                ),
            ),
        ),
        all(
            target_arch = "powerpc64",
            portable_atomic_unstable_asm_experimental_arch,
            any(
                target_feature = "quadword-atomics",
                portable_atomic_target_feature = "quadword-atomics",
                all(
                    feature = "fallback",
                    not(portable_atomic_no_outline_atomics),
                    portable_atomic_outline_atomics, // TODO(powerpc64): currently disabled by default
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
                    ),
                    not(any(miri, portable_atomic_sanitize_thread)),
                ),
            ),
        ),
        all(target_arch = "s390x", portable_atomic_unstable_asm_experimental_arch),
    )))]
    pub(crate) use self::fallback::{AtomicI128, AtomicU128};
}

// 64-bit atomics (platform-specific)
// pre-v6 ARM Linux
#[cfg(feature = "fallback")]
#[cfg(all(
    target_arch = "arm",
    not(any(miri, portable_atomic_sanitize_thread)),
    not(portable_atomic_no_asm),
    any(target_os = "linux", target_os = "android"),
    not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
    not(portable_atomic_no_outline_atomics),
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_64))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "64")))]
pub(crate) use self::arm_linux::{AtomicI64, AtomicU64};

// 128-bit atomics (platform-specific)
// aarch64
#[cfg(all(
    target_arch = "aarch64",
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
))]
pub(crate) use self::aarch64::{AtomicI128, AtomicU128};
// x86_64 & (cmpxchg16b | outline-atomics)
#[cfg(all(
    target_arch = "x86_64",
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    any(
        target_feature = "cmpxchg16b",
        portable_atomic_target_feature = "cmpxchg16b",
        all(
            feature = "fallback",
            not(portable_atomic_no_cmpxchg16b_target_feature),
            not(portable_atomic_no_outline_atomics),
            not(any(target_env = "sgx", miri)),
        ),
    ),
))]
pub(crate) use self::x86_64::{AtomicI128, AtomicU128};
// powerpc64 & (pwr8 | outline-atomics)
#[cfg(all(
    target_arch = "powerpc64",
    portable_atomic_unstable_asm_experimental_arch,
    any(
        target_feature = "quadword-atomics",
        portable_atomic_target_feature = "quadword-atomics",
        all(
            feature = "fallback",
            not(portable_atomic_no_outline_atomics),
            portable_atomic_outline_atomics, // TODO(powerpc64): currently disabled by default
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
            ),
            not(any(miri, portable_atomic_sanitize_thread)),
        ),
    ),
))]
pub(crate) use self::powerpc64::{AtomicI128, AtomicU128};
// s390x
#[cfg(all(target_arch = "s390x", portable_atomic_unstable_asm_experimental_arch))]
pub(crate) use self::s390x::{AtomicI128, AtomicU128};
