#![cfg_attr(not(all(test, feature = "float")), allow(dead_code, unused_macros))]

#[macro_use]
#[path = "gen/utils.rs"]
mod gen;

use core::sync::atomic::Ordering;

macro_rules! static_assert {
    ($cond:expr $(,)?) => {{
        let [] = [(); true as usize - $crate::utils::_assert_is_bool($cond) as usize];
    }};
}
pub(crate) const fn _assert_is_bool(v: bool) -> bool {
    v
}

macro_rules! static_assert_layout {
    ($atomic_type:ty, $value_type:ty) => {
        static_assert!(
            core::mem::align_of::<$atomic_type>() == core::mem::size_of::<$atomic_type>()
        );
        static_assert!(core::mem::size_of::<$atomic_type>() == core::mem::size_of::<$value_type>());
    };
}

// #[doc = concat!(...)] requires Rust 1.54
macro_rules! doc_comment {
    ($doc:expr, $($tt:tt)*) => {
        #[doc = $doc]
        $($tt)*
    };
}

// Adapted from https://github.com/BurntSushi/memchr/blob/2.4.1/src/memchr/x86/mod.rs#L9-L71.
/// # Safety
///
/// - the caller must uphold the safety contract for the function returned by $detect_body.
/// - the memory pointed by the function pointer returned by $detect_body must be visible from any threads.
///
/// The second requirement is always met if the function pointer is to the function definition.
/// (Currently, all uses of this macro in our code are in this case.)
#[allow(unused_macros)]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "powerpc64",
    all(target_arch = "x86_64", not(any(target_env = "sgx", miri))),
))]
macro_rules! ifunc {
    (unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)? { $($detect_body:tt)* }) => {{
        type FnTy = unsafe fn($($arg_ty),*) $(-> $ret_ty)?;
        static FUNC: core::sync::atomic::AtomicPtr<()>
            = core::sync::atomic::AtomicPtr::new(detect as *mut ());
        #[cold]
        unsafe fn detect($($arg_pat: $arg_ty),*) $(-> $ret_ty)? {
            let func: FnTy = { $($detect_body)* };
            FUNC.store(func as *mut (), core::sync::atomic::Ordering::Relaxed);
            // SAFETY: the caller must uphold the safety contract for the function returned by $detect_body.
            unsafe { func($($arg_pat),*) }
        }
        // SAFETY: `FnTy` is a function pointer, which is always safe to transmute with a `*mut ()`.
        // (To force the caller to use unsafe block for this macro, do not use
        // unsafe block here.)
        let func = {
            core::mem::transmute::<*mut (), FnTy>(FUNC.load(core::sync::atomic::Ordering::Relaxed))
        };
        // SAFETY: the caller must uphold the safety contract for the function returned by $detect_body.
        // (To force the caller to use unsafe block for this macro, do not use
        // unsafe block here.)
        func($($arg_pat),*)
    }};
}

#[allow(unused_macros)]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "powerpc64",
    all(target_arch = "x86_64", not(any(target_env = "sgx", miri))),
))]
macro_rules! fn_alias {
    (
        $(#[$($fn_attr:tt)*])*
        $vis:vis unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)?;
        $(#[$($alias_attr:tt)*])*
        $new:ident = $from:ident($($last_args:tt)*);
        $($rest:tt)*
    ) => {
        $(#[$($fn_attr)*])*
        $(#[$($alias_attr)*])*
        $vis unsafe fn $new($($arg_pat: $arg_ty),*) $(-> $ret_ty)? {
            // SAFETY: the caller must uphold the safety contract.
            unsafe { $from($($arg_pat,)* $($last_args)*) }
        }
        fn_alias! {
            $(#[$($fn_attr)*])*
            $vis unsafe fn($($arg_pat: $arg_ty),*) $(-> $ret_ty)?;
            $($rest)*
        }
    };
    (
        $(#[$($attr:tt)*])*
        $vis:vis unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)?;
    ) => {}
}

/// Make the given function const if the given condition is true.
macro_rules! const_fn {
    (
        const_if: #[cfg($($cfg:tt)+)];
        $(#[$($attr:tt)*])*
        $vis:vis const fn $($rest:tt)*
    ) => {
        #[cfg($($cfg)+)]
        $(#[$($attr)*])*
        $vis const fn $($rest)*
        #[cfg(not($($cfg)+))]
        $(#[$($attr)*])*
        $vis fn $($rest)*
    };
}

/// Implements `core::fmt::Debug` and `serde::{Serialize, Deserialize}` (when serde
/// feature is enabled) for atomic bool, integer, or float.
macro_rules! impl_debug_and_serde {
    ($atomic_type:ident) => {
        impl fmt::Debug for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // fmt is not hot path
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // std atomic types use Relaxed in Debug::fmt: https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L2024
                fmt::Debug::fmt(&self.load(Ordering::Relaxed), f)
            }
        }
        #[cfg(feature = "serde")]
        #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
        impl serde::Serialize for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // serde doesn't use inline on std atomic's Serialize/Deserialize impl
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                // https://github.com/serde-rs/serde/blob/v1.0.152/serde/src/ser/impls.rs#L958-L959
                self.load(Ordering::Relaxed).serialize(serializer)
            }
        }
        #[cfg(feature = "serde")]
        #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
        impl<'de> serde::Deserialize<'de> for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // serde doesn't use inline on std atomic's Serialize/Deserialize impl
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer).map(Self::new)
            }
        }
    };
}

// We do not provide `nand` because it cannot be optimized on neither x86 nor MSP430.
// https://godbolt.org/z/x88voWGov
macro_rules! impl_default_no_fetch_ops {
    ($atomic_type:ident, bool) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn and(&self, val: bool, order: Ordering) {
                self.fetch_and(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn or(&self, val: bool, order: Ordering) {
                self.fetch_or(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn xor(&self, val: bool, order: Ordering) {
                self.fetch_xor(val, order);
            }
        }
    };
    ($atomic_type:ident, $int_type:ident) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn add(&self, val: $int_type, order: Ordering) {
                self.fetch_add(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn sub(&self, val: $int_type, order: Ordering) {
                self.fetch_sub(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn and(&self, val: $int_type, order: Ordering) {
                self.fetch_and(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn or(&self, val: $int_type, order: Ordering) {
                self.fetch_or(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn xor(&self, val: $int_type, order: Ordering) {
                self.fetch_xor(val, order);
            }
        }
    };
}
macro_rules! impl_default_bit_opts {
    ($atomic_type:ident, $int_type:ident) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_set(&self, bit: u32, order: Ordering) -> bool {
                let mask = (1 as $int_type).wrapping_shl(bit);
                self.fetch_or(mask, order) & mask != 0
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_clear(&self, bit: u32, order: Ordering) -> bool {
                let mask = (1 as $int_type).wrapping_shl(bit);
                self.fetch_and(!mask, order) & mask != 0
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_toggle(&self, bit: u32, order: Ordering) -> bool {
                let mask = (1 as $int_type).wrapping_shl(bit);
                self.fetch_xor(mask, order) & mask != 0
            }
        }
    };
}

#[cfg(not(all(
    target_arch = "mips",
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
)))]
#[macro_use]
mod atomic_ptr_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_ptr {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg(all(
    target_arch = "mips",
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
))]
#[macro_use]
mod atomic_ptr_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_ptr {
        ($($tt:tt)*) => {};
    }
}

#[cfg(not(all(
    any(target_arch = "mips", target_arch = "bpf"),
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
)))]
#[macro_use]
mod atomic_8_16_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_8 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_16 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg(all(
    any(target_arch = "mips", target_arch = "bpf"),
    portable_atomic_no_atomic_load_store,
    not(feature = "critical-section"),
))]
#[macro_use]
mod atomic_8_16_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_8 {
        ($($tt:tt)*) => {};
    }
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_16 {
        ($($tt:tt)*) => {};
    }
}

#[cfg(all(
    any(not(target_pointer_width = "16"), feature = "fallback"),
    not(all(
        any(target_arch = "mips", target_arch = "bpf"),
        portable_atomic_no_atomic_load_store,
        not(feature = "critical-section"),
    )),
))]
#[macro_use]
mod atomic_32_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_32 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg(not(all(
    any(not(target_pointer_width = "16"), feature = "fallback"),
    not(all(
        any(target_arch = "mips", target_arch = "bpf"),
        portable_atomic_no_atomic_load_store,
        not(feature = "critical-section"),
    )),
)))]
#[macro_use]
mod atomic_32_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_32 {
        ($($tt:tt)*) => {};
    }
}

#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(any(
        all(
            feature = "fallback",
            any(
                not(portable_atomic_no_atomic_cas),
                portable_atomic_unsafe_assume_single_core,
                feature = "critical-section",
                target_arch = "avr",
                target_arch = "msp430",
            ),
        ),
        not(portable_atomic_no_atomic_64),
        not(any(target_pointer_width = "16", target_pointer_width = "32")),
    ))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(
        all(
            feature = "fallback",
            any(
                target_has_atomic = "ptr",
                portable_atomic_unsafe_assume_single_core,
                feature = "critical-section",
                target_arch = "avr",
                target_arch = "msp430",
            ),
        ),
        target_has_atomic = "64",
        not(any(target_pointer_width = "16", target_pointer_width = "32")),
    ))
)]
#[macro_use]
mod atomic_64_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_64 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(any(
        all(
            feature = "fallback",
            any(
                not(portable_atomic_no_atomic_cas),
                portable_atomic_unsafe_assume_single_core,
                feature = "critical-section",
                target_arch = "avr",
                target_arch = "msp430",
            ),
        ),
        not(portable_atomic_no_atomic_64),
        not(any(target_pointer_width = "16", target_pointer_width = "32")),
    )))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(any(
        all(
            feature = "fallback",
            any(
                target_has_atomic = "ptr",
                portable_atomic_unsafe_assume_single_core,
                feature = "critical-section",
                target_arch = "avr",
                target_arch = "msp430",
            ),
        ),
        target_has_atomic = "64",
        not(any(target_pointer_width = "16", target_pointer_width = "32")),
    )))
)]
#[macro_use]
mod atomic_64_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_64 {
        ($($tt:tt)*) => {};
    }
}

#[cfg_attr(
    not(feature = "fallback"),
    cfg(any(
        all(
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            target_arch = "aarch64",
        ),
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
    ))
)]
#[cfg_attr(
    all(feature = "fallback", portable_atomic_no_cfg_target_has_atomic),
    cfg(any(
        not(portable_atomic_no_atomic_cas),
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    ))
)]
#[cfg_attr(
    all(feature = "fallback", not(portable_atomic_no_cfg_target_has_atomic)),
    cfg(any(
        target_has_atomic = "ptr",
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    ))
)]
#[macro_use]
mod atomic_128_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_128 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg_attr(
    not(feature = "fallback"),
    cfg(not(any(
        all(
            any(not(portable_atomic_no_asm), portable_atomic_unstable_asm),
            target_arch = "aarch64",
        ),
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
    )))
)]
#[cfg_attr(
    all(feature = "fallback", portable_atomic_no_cfg_target_has_atomic),
    cfg(not(any(
        not(portable_atomic_no_atomic_cas),
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    )))
)]
#[cfg_attr(
    all(feature = "fallback", not(portable_atomic_no_cfg_target_has_atomic)),
    cfg(not(any(
        target_has_atomic = "ptr",
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    )))
)]
#[macro_use]
mod atomic_128_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_128 {
        ($($tt:tt)*) => {};
    }
}

#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(any(
        not(portable_atomic_no_atomic_cas),
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    ))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(
        target_has_atomic = "ptr",
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    ))
)]
#[macro_use]
mod atomic_cas_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_cas {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
}
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(not(any(
        not(portable_atomic_no_atomic_cas),
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    )))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(any(
        target_has_atomic = "ptr",
        portable_atomic_unsafe_assume_single_core,
        feature = "critical-section",
        target_arch = "avr",
        target_arch = "msp430",
    )))
)]
#[macro_use]
mod atomic_cas_macros {
    #[doc(hidden)] // Not public API. (please submit an issue if you want this to be public API)
    #[macro_export]
    macro_rules! cfg_has_atomic_cas {
        ($($tt:tt)*) => {};
    }
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3155
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_load_ordering(order: Ordering) {
    match order {
        Ordering::Acquire | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Release => panic!("there is no such thing as a release load"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release load"),
        _ => unreachable!("{:?}", order),
    }
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3140
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_store_ordering(order: Ordering) {
    match order {
        Ordering::Release | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Acquire => panic!("there is no such thing as an acquire store"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release store"),
        _ => unreachable!("{:?}", order),
    }
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3221
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_compare_exchange_ordering(success: Ordering, failure: Ordering) {
    match success {
        Ordering::AcqRel
        | Ordering::Acquire
        | Ordering::Relaxed
        | Ordering::Release
        | Ordering::SeqCst => {}
        _ => unreachable!("{:?}, {:?}", success, failure),
    }
    match failure {
        Ordering::Acquire | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Release => panic!("there is no such thing as a release failure ordering"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release failure ordering"),
        _ => unreachable!("{:?}, {:?}", success, failure),
    }
}

// https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2016/p0418r2.html
// https://github.com/rust-lang/rust/pull/98383
#[allow(dead_code)]
#[inline]
pub(crate) fn upgrade_success_ordering(success: Ordering, failure: Ordering) -> Ordering {
    match (success, failure) {
        (Ordering::Relaxed, Ordering::Acquire) => Ordering::Acquire,
        (Ordering::Release, Ordering::Acquire) => Ordering::AcqRel,
        (_, Ordering::SeqCst) => Ordering::SeqCst,
        _ => success,
    }
}

/// Emulate strict provenance.
///
/// Once strict_provenance is stable, migrate to the standard library's APIs.
#[cfg(miri)]
#[allow(
    clippy::cast_possible_wrap,
    clippy::transmutes_expressible_as_ptr_casts,
    clippy::useless_transmute
)]
pub(crate) mod strict {
    use core::mem;

    /// Get the address of a pointer.
    #[inline]
    #[must_use]
    pub(crate) fn addr<T>(ptr: *mut T) -> usize {
        // SAFETY: Every sized pointer is a valid integer for the time being.
        unsafe { mem::transmute(ptr) }
    }

    /// Replace the address portion of this pointer with a new address.
    #[inline]
    #[must_use]
    pub(crate) fn with_addr<T>(ptr: *mut T, addr: usize) -> *mut T {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        //
        // In the mean-time, this operation is defined to be "as if" it was
        // a wrapping_offset, so we can emulate it as such. This should properly
        // restore pointer provenance even under today's compiler.
        let self_addr = self::addr(ptr) as isize;
        let dest_addr = addr as isize;
        let offset = dest_addr.wrapping_sub(self_addr);

        // This is the canonical desugaring of this operation.
        (ptr as *mut u8).wrapping_offset(offset) as *mut T
    }

    /// Run an operation of some kind on a pointer.
    #[inline]
    #[must_use]
    pub(crate) fn map_addr<T>(ptr: *mut T, f: impl FnOnce(usize) -> usize) -> *mut T {
        self::with_addr(ptr, f(addr(ptr)))
    }
}
