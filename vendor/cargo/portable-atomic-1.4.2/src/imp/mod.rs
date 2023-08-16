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
    cfg(not(all(feature = "critical-section", portable_atomic_no_atomic_cas)))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        not(target_has_atomic = "ptr"),
    )))
)]
mod core_atomic;

#[cfg(any(not(portable_atomic_no_asm), portable_atomic_unstable_asm))]
#[cfg(target_arch = "aarch64")]
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

#[cfg(any(not(portable_atomic_no_asm), portable_atomic_unstable_asm))]
#[cfg(target_arch = "x86_64")]
#[cfg(any(
    target_feature = "cmpxchg16b",
    portable_atomic_target_feature = "cmpxchg16b",
    all(
        feature = "fallback",
        not(portable_atomic_no_cmpxchg16b_target_feature),
        not(portable_atomic_no_outline_atomics),
        not(any(target_env = "sgx", miri)),
    ),
))]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(any(miri, portable_atomic_sanitize_thread), path = "atomic128/intrinsics.rs")]
#[cfg_attr(not(any(miri, portable_atomic_sanitize_thread)), path = "atomic128/x86_64.rs")]
mod x86_64;

#[cfg(portable_atomic_unstable_asm_experimental_arch)]
#[cfg(target_arch = "powerpc64")]
#[cfg(any(
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
                    all(target_env = "musl", not(target_feature = "crt-static")),
                    portable_atomic_outline_atomics,
                ),
            ),
            target_os = "freebsd",
        ),
        not(any(miri, portable_atomic_sanitize_thread)),
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

#[cfg(portable_atomic_unstable_asm_experimental_arch)]
#[cfg(target_arch = "s390x")]
// Use intrinsics.rs on Miri and Sanitizer that do not support inline assembly.
#[cfg_attr(
    all(any(miri, portable_atomic_sanitize_thread), portable_atomic_new_atomic_intrinsics),
    path = "atomic128/intrinsics.rs"
)]
#[cfg_attr(
    not(all(any(miri, portable_atomic_sanitize_thread), portable_atomic_new_atomic_intrinsics)),
    path = "atomic128/s390x.rs"
)]
mod s390x;

// Miri and Sanitizer do not support inline assembly.
#[cfg(feature = "fallback")]
#[cfg(all(
    not(any(miri, portable_atomic_sanitize_thread)),
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    target_arch = "arm",
    any(target_os = "linux", target_os = "android"),
    not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
    not(portable_atomic_no_outline_atomics),
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_64))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "64")))]
mod arm_linux;

#[cfg(target_arch = "msp430")]
pub(crate) mod msp430;

#[cfg(any(test, not(feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(any(test, portable_atomic_no_atomic_cas)))]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(test, not(target_has_atomic = "ptr")))
)]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod riscv;

// Miri and Sanitizer do not support inline assembly.
#[cfg(all(
    not(any(miri, portable_atomic_sanitize_thread)),
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    any(target_arch = "x86", target_arch = "x86_64"),
))]
mod x86;

// -----------------------------------------------------------------------------
// Lock-based fallback implementations

#[cfg(feature = "fallback")]
#[cfg(any(
    test,
    not(any(
        all(
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            target_arch = "aarch64",
        ),
        all(
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            any(target_feature = "cmpxchg16b", portable_atomic_target_feature = "cmpxchg16b"),
            target_arch = "x86_64",
        ),
        all(
            portable_atomic_unstable_asm_experimental_arch,
            any(
                target_feature = "quadword-atomics",
                portable_atomic_target_feature = "quadword-atomics",
            ),
            target_arch = "powerpc64",
        ),
        all(portable_atomic_unstable_asm_experimental_arch, target_arch = "s390x"),
    ))
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(not(portable_atomic_no_atomic_cas)))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(target_has_atomic = "ptr"))]
mod fallback;

// -----------------------------------------------------------------------------
// Critical section based fallback implementations

// On AVR, we always use critical section based fallback implementation.
// AVR can be safely assumed to be single-core, so this is sound.
// https://github.com/llvm/llvm-project/blob/llvmorg-16.0.0/llvm/lib/Target/AVR/AVRExpandPseudoInsts.cpp#LL963
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
    feature = "critical-section",
    target_arch = "arm",
    target_arch = "avr",
    target_arch = "msp430",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "xtensa",
))]
mod interrupt;

// -----------------------------------------------------------------------------
// Atomic float implementations

#[cfg(feature = "float")]
pub(crate) mod float;

// -----------------------------------------------------------------------------

// Atomic{Isize,Usize,Bool,Ptr}, Atomic{I,U}{8,16}
#[cfg(not(any(
    portable_atomic_no_atomic_load_store,
    portable_atomic_unsafe_assume_single_core,
    target_arch = "avr",
    target_arch = "msp430",
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(all(feature = "critical-section", portable_atomic_no_atomic_cas)))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        not(target_has_atomic = "ptr"),
    )))
)]
pub(crate) use self::core_atomic::{
    AtomicI16, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU8, AtomicUsize,
};
// RISC-V without A-extension
#[cfg(not(any(portable_atomic_unsafe_assume_single_core, feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub(crate) use self::riscv::{
    AtomicI16, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU8, AtomicUsize,
};
// no core Atomic{Isize,Usize,Bool,Ptr}/Atomic{I,U}{8,16} & assume single core => critical section based fallback
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
pub(crate) use self::interrupt::{
    AtomicI16, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU8, AtomicUsize,
};
// bpf
#[cfg(all(
    target_arch = "bpf",
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
))]
pub(crate) use self::core_atomic::{AtomicI64, AtomicIsize, AtomicPtr, AtomicU64, AtomicUsize};

// Atomic{I,U}32
#[cfg(not(any(
    portable_atomic_no_atomic_load_store,
    portable_atomic_unsafe_assume_single_core,
    target_arch = "avr",
    target_arch = "msp430",
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(all(feature = "critical-section", portable_atomic_no_atomic_cas)))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(all(
        any(target_arch = "riscv32", target_arch = "riscv64", feature = "critical-section"),
        not(target_has_atomic = "ptr"),
    )))
)]
pub(crate) use self::core_atomic::{AtomicI32, AtomicU32};
// RISC-V without A-extension
#[cfg(not(any(portable_atomic_unsafe_assume_single_core, feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub(crate) use self::riscv::{AtomicI32, AtomicU32};
// no core Atomic{I,U}32 & no CAS & assume single core => critical section based fallback
#[cfg(any(not(target_pointer_width = "16"), feature = "fallback"))]
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
pub(crate) use self::interrupt::{AtomicI32, AtomicU32};

// Atomic{I,U}64
#[cfg(not(any(
    portable_atomic_no_atomic_load_store,
    portable_atomic_unsafe_assume_single_core,
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(any(
        not(portable_atomic_no_atomic_64),
        all(
            not(any(target_pointer_width = "16", target_pointer_width = "32")),
            not(all(feature = "critical-section", portable_atomic_no_atomic_cas)),
        ),
    ))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(
        target_has_atomic = "64",
        all(
            not(any(target_pointer_width = "16", target_pointer_width = "32")),
            not(all(
                any(
                    target_arch = "riscv32",
                    target_arch = "riscv64",
                    feature = "critical-section",
                ),
                not(target_has_atomic = "ptr"),
            )),
        ),
    ))
)]
pub(crate) use self::core_atomic::{AtomicI64, AtomicU64};
// RISC-V without A-extension
#[cfg(not(any(portable_atomic_unsafe_assume_single_core, feature = "critical-section")))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv::{AtomicI64, AtomicU64};
// pre-v6 ARM Linux
#[cfg(feature = "fallback")]
#[cfg(all(
    not(any(miri, portable_atomic_sanitize_thread)),
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    target_arch = "arm",
    any(target_os = "linux", target_os = "android"),
    not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
    not(portable_atomic_no_outline_atomics),
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_64))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "64")))]
pub(crate) use self::arm_linux::{AtomicI64, AtomicU64};
// no core Atomic{I,U}64 & has CAS => use lock-base fallback
#[cfg(feature = "fallback")]
#[cfg(not(all(
    not(any(miri, portable_atomic_sanitize_thread)),
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    target_arch = "arm",
    any(target_os = "linux", target_os = "android"),
    not(any(target_feature = "v6", portable_atomic_target_feature = "v6")),
    not(portable_atomic_no_outline_atomics),
)))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(all(portable_atomic_no_atomic_64, not(portable_atomic_no_atomic_cas)))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(all(not(target_has_atomic = "64"), target_has_atomic = "ptr"))
)]
pub(crate) use self::fallback::{AtomicI64, AtomicU64};
// no core Atomic{I,U}64 & no CAS & assume single core => critical section based fallback
#[cfg(any(
    not(any(target_pointer_width = "16", target_pointer_width = "32")),
    feature = "fallback",
))]
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
pub(crate) use self::interrupt::{AtomicI64, AtomicU64};

// Atomic{I,U}128
// aarch64 stable
#[cfg(all(
    any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
    target_arch = "aarch64",
))]
pub(crate) use self::aarch64::{AtomicI128, AtomicU128};
// no core Atomic{I,U}128 & has cmpxchg16b => use cmpxchg16b
#[cfg(all(
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
    target_arch = "x86_64",
))]
pub(crate) use self::x86_64::{AtomicI128, AtomicU128};
// powerpc64
#[cfg(portable_atomic_unstable_asm_experimental_arch)]
#[cfg(any(
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
                    all(target_env = "musl", not(target_feature = "crt-static")),
                    portable_atomic_outline_atomics,
                ),
            ),
            target_os = "freebsd",
        ),
        not(any(miri, portable_atomic_sanitize_thread)),
    ),
))]
#[cfg(target_arch = "powerpc64")]
pub(crate) use self::powerpc64::{AtomicI128, AtomicU128};
// s390x
#[cfg(portable_atomic_unstable_asm_experimental_arch)]
#[cfg(target_arch = "s390x")]
pub(crate) use self::s390x::{AtomicI128, AtomicU128};
// no core Atomic{I,U}128 & has CAS => use lock-base fallback
#[cfg(feature = "fallback")]
#[cfg(not(any(
    all(any(not(portable_atomic_no_asm), portable_atomic_unstable_asm), target_arch = "aarch64"),
    all(
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
        target_arch = "x86_64",
    ),
    all(
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
                            all(target_env = "musl", not(target_feature = "crt-static")),
                            portable_atomic_outline_atomics,
                        ),
                    ),
                    target_os = "freebsd",
                ),
                not(any(miri, portable_atomic_sanitize_thread)),
            ),
        ),
        target_arch = "powerpc64",
    ),
    all(portable_atomic_unstable_asm_experimental_arch, target_arch = "s390x"),
)))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(not(portable_atomic_no_atomic_cas)))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(target_has_atomic = "ptr"))]
pub(crate) use self::fallback::{AtomicI128, AtomicU128};
// no core Atomic{I,U}128 & no CAS & assume_single_core => critical section based fallback
#[cfg(feature = "fallback")]
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(portable_atomic_no_cfg_target_has_atomic, cfg(portable_atomic_no_atomic_cas))]
#[cfg_attr(not(portable_atomic_no_cfg_target_has_atomic), cfg(not(target_has_atomic = "ptr")))]
pub(crate) use self::interrupt::{AtomicI128, AtomicU128};
