// SPDX-License-Identifier: Apache-2.0 OR MIT

// Run-time feature detection on aarch64 Windows by using IsProcessorFeaturePresent.
//
// As of nightly-2023-01-23, is_aarch64_feature_detected doesn't support run-time detection of FEAT_LSE on Windows.
// https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/std_detect/src/detect/os/windows/aarch64.rs
// https://github.com/rust-lang/stdarch/pull/1373
//
// Refs: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-isprocessorfeaturepresent

include!("common.rs");

// windows-sys requires Rust 1.56
#[allow(clippy::upper_case_acronyms)]
mod ffi {
    pub(crate) type DWORD = u32;
    pub(crate) type BOOL = i32;

    pub(crate) const FALSE: BOOL = 0;
    // Defined in winnt.h of Windows SDK.
    pub(crate) const PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE: DWORD = 34;

    extern "system" {
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-isprocessorfeaturepresent
        pub(crate) fn IsProcessorFeaturePresent(ProcessorFeature: DWORD) -> BOOL;
    }
}

#[cold]
fn _detect(info: &mut CpuInfo) {
    // SAFETY: calling IsProcessorFeaturePresent is safe, and FALSE is also
    // returned if the HAL does not support detection of the specified feature.
    if unsafe {
        ffi::IsProcessorFeaturePresent(ffi::PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE) != ffi::FALSE
    } {
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

    // Static assertions for FFI bindings.
    // This checks that FFI bindings defined in this crate and FFI bindings defined
    // in windows-sys have compatible signatures (or the same values if constants).
    // Since this is static assertion, we can detect problems with
    // `cargo check --tests --target <target>` run in CI (via TESTS=1 build.sh)
    // without actually running tests on these platforms.
    // (Unlike libc, windows-sys programmatically generates bindings from Windows
    // API metadata, so it should be enough to check compatibility with the
    // windows-sys' signatures/values.)
    // See also tools/codegen/src/ffi.rs.
    // TODO(codegen): auto-generate this test
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::no_effect_underscore_binding
    )]
    const _: fn() = || {
        use test_helper::windows_sys;
        let _: ffi::DWORD = 0 as windows_sys::Win32::System::Threading::PROCESSOR_FEATURE_ID;
        let _: ffi::BOOL = 0 as windows_sys::Win32::Foundation::BOOL;
        let mut _sysctl: unsafe extern "system" fn(ffi::DWORD) -> ffi::BOOL =
            ffi::IsProcessorFeaturePresent;
        _sysctl = windows_sys::Win32::System::Threading::IsProcessorFeaturePresent;
        static_assert!(ffi::FALSE == windows_sys::Win32::Foundation::FALSE);
        static_assert!(
            ffi::PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE
                == windows_sys::Win32::System::Threading::PF_ARM_V81_ATOMIC_INSTRUCTIONS_AVAILABLE
        );
    };
}
