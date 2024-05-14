// SPDX-License-Identifier: Apache-2.0 OR MIT

// Run-time feature detection on aarch64 Fuchsia by using zx_system_get_features.
//
// As of nightly-2023-01-23, is_aarch64_feature_detected doesn't support run-time detection on Fuchsia.
// https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/std_detect/src/detect/mod.rs
//
// Refs:
// - https://fuchsia.dev/fuchsia-src/reference/syscalls/system_get_features
// - https://github.com/llvm/llvm-project/commit/4e731abc55681751b5d736b613f7720e50eb1ad4

include!("common.rs");

#[allow(non_camel_case_types)]
mod ffi {
    // https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/zircon/system/public/zircon/types.h
    pub(crate) type zx_status_t = i32;

    // https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/zircon/system/public/zircon/errors.h
    pub(crate) const ZX_OK: zx_status_t = 0;
    // https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/zircon/system/public/zircon/features.h
    pub(crate) const ZX_FEATURE_KIND_CPU: u32 = 0;
    pub(crate) const ZX_ARM64_FEATURE_ISA_ATOMICS: u32 = 1 << 8;

    #[link(name = "zircon")]
    extern "C" {
        // https://fuchsia.dev/fuchsia-src/reference/syscalls/system_get_features
        pub(crate) fn zx_system_get_features(kind: u32, features: *mut u32) -> zx_status_t;
    }
}

fn zx_system_get_features(kind: u32) -> u32 {
    let mut out = 0_u32;
    // SAFETY: the pointer is valid because we got it from a reference.
    let res = unsafe { ffi::zx_system_get_features(kind, &mut out) };
    if res != ffi::ZX_OK {
        return 0;
    }
    out
}

#[cold]
fn _detect(info: &mut CpuInfo) {
    let features = zx_system_get_features(ffi::ZX_FEATURE_KIND_CPU);
    if features & ffi::ZX_ARM64_FEATURE_ISA_ATOMICS != 0 {
        info.set(CpuInfo::HAS_LSE);
    }
}

#[allow(
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::undocumented_unsafe_blocks,
    clippy::wildcard_imports
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuchsia() {
        let features = zx_system_get_features(ffi::ZX_FEATURE_KIND_CPU);
        assert_ne!(features, 0);
        std::eprintln!("features: {:b}", features);
    }

    // Static assertions for FFI bindings.
    // This checks that FFI bindings defined in this crate and FFI bindings
    // generated for the platform's latest header file using bindgen have
    // compatible signatures (or the same values if constants).
    // Since this is static assertion, we can detect problems with
    // `cargo check --tests --target <target>` run in CI (via TESTS=1 build.sh)
    // without actually running tests on these platforms.
    // See also tools/codegen/src/ffi.rs.
    // TODO(codegen): auto-generate this test
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::no_effect_underscore_binding
    )]
    const _: fn() = || {
        use test_helper::sys;
        // TODO(codegen): zx_system_get_features
        let _: ffi::zx_status_t = 0 as sys::zx_status_t;
        static_assert!(ffi::ZX_OK == sys::ZX_OK as ffi::zx_status_t);
        static_assert!(ffi::ZX_FEATURE_KIND_CPU == sys::ZX_FEATURE_KIND_CPU);
        static_assert!(ffi::ZX_ARM64_FEATURE_ISA_ATOMICS == sys::ZX_ARM64_FEATURE_ISA_ATOMICS);
    };
}
