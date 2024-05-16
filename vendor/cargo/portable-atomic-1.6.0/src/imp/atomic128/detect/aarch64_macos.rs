// SPDX-License-Identifier: Apache-2.0 OR MIT

// Run-time feature detection on aarch64 macOS by using sysctl.
//
// This module is currently only enabled on tests because aarch64 macOS always supports FEAT_LSE and FEAT_LSE2.
// https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/include/llvm/TargetParser/AArch64TargetParser.h#L494
//
// If macOS supporting Armv9.4-a becomes popular in the future, this module will
// be used to support outline-atomics for FEAT_LSE128/FEAT_LRCPC3.
//
// Refs: https://developer.apple.com/documentation/kernel/1387446-sysctlbyname/determining_instruction_set_characteristics
//
// Note that iOS doesn't support sysctl:
// - https://developer.apple.com/forums/thread/9440
// - https://nabla-c0d3.github.io/blog/2015/06/16/ios9-security-privacy

include!("common.rs");

use core::ptr;

// core::ffi::c_* (except c_void) requires Rust 1.64, libc will soon require Rust 1.47
#[allow(non_camel_case_types)]
mod ffi {
    pub(crate) use super::c_types::{c_char, c_int, c_size_t, c_void};

    extern "C" {
        // https://developer.apple.com/documentation/kernel/1387446-sysctlbyname
        // https://github.com/apple-oss-distributions/xnu/blob/5c2921b07a2480ab43ec66f5b9e41cb872bc554f/bsd/sys/sysctl.h
        // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/bsd/apple/mod.rs#L5167-L5173
        pub(crate) fn sysctlbyname(
            name: *const c_char,
            old_p: *mut c_void,
            old_len_p: *mut c_size_t,
            new_p: *mut c_void,
            new_len: c_size_t,
        ) -> c_int;
    }
}

unsafe fn sysctlbyname32(name: &[u8]) -> Option<u32> {
    const OUT_LEN: ffi::c_size_t = core::mem::size_of::<u32>() as ffi::c_size_t;

    debug_assert_eq!(name.last(), Some(&0), "{:?}", name);
    debug_assert_eq!(name.iter().filter(|&&v| v == 0).count(), 1, "{:?}", name);

    let mut out = 0_u32;
    let mut out_len = OUT_LEN;
    // SAFETY:
    // - the caller must guarantee that `name` a valid C string.
    // - `out_len` does not exceed the size of `out`.
    // - `sysctlbyname` is thread-safe.
    let res = unsafe {
        ffi::sysctlbyname(
            name.as_ptr().cast::<ffi::c_char>(),
            (&mut out as *mut u32).cast::<ffi::c_void>(),
            &mut out_len,
            ptr::null_mut(),
            0,
        )
    };
    if res != 0 {
        return None;
    }
    debug_assert_eq!(out_len, OUT_LEN);
    Some(out)
}

#[cold]
fn _detect(info: &mut CpuInfo) {
    // hw.optional.armv8_1_atomics is available on macOS 11+ (note: aarch64 support was added on macOS 11),
    // hw.optional.arm.FEAT_* are only available on macOS 12+.
    // Query both names in case future versions of macOS remove the old name.
    // https://github.com/golang/go/commit/c15593197453b8bf90fc3a9080ba2afeaf7934ea
    // https://github.com/google/boringssl/commit/91e0b11eba517d83b910b20fe3740eeb39ecb37e
    // SAFETY: we passed a valid C string.
    if unsafe {
        sysctlbyname32(b"hw.optional.arm.FEAT_LSE\0").unwrap_or(0) != 0
            || sysctlbyname32(b"hw.optional.armv8_1_atomics\0").unwrap_or(0) != 0
    } {
        info.set(CpuInfo::HAS_LSE);
    }
    // SAFETY: we passed a valid C string.
    if unsafe { sysctlbyname32(b"hw.optional.arm.FEAT_LSE2\0").unwrap_or(0) != 0 } {
        info.set(CpuInfo::HAS_LSE2);
    }
    // we currently only use FEAT_LSE and FEAT_LSE2 in outline-atomics.
    #[cfg(test)]
    {
        // SAFETY: we passed a valid C string.
        if unsafe { sysctlbyname32(b"hw.optional.arm.FEAT_LSE128\0").unwrap_or(0) != 0 } {
            info.set(CpuInfo::HAS_LSE128);
        }
        // SAFETY: we passed a valid C string.
        if unsafe { sysctlbyname32(b"hw.optional.arm.FEAT_LRCPC3\0").unwrap_or(0) != 0 } {
            info.set(CpuInfo::HAS_RCPC3);
        }
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
    fn test_macos() {
        unsafe {
            assert_eq!(sysctlbyname32(b"hw.optional.armv8_1_atomics\0"), Some(1));
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LSE\0"), Some(1));
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LSE2\0"), Some(1));
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LSE128\0"), None);
            assert_eq!(std::io::Error::last_os_error().kind(), std::io::ErrorKind::NotFound);
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LRCPC\0"), Some(1));
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LRCPC2\0"), Some(1));
            assert_eq!(sysctlbyname32(b"hw.optional.arm.FEAT_LRCPC3\0"), None);
            assert_eq!(std::io::Error::last_os_error().kind(), std::io::ErrorKind::NotFound);
        }
    }

    // Static assertions for FFI bindings.
    // This checks that FFI bindings defined in this crate, FFI bindings defined
    // in libc, and FFI bindings generated for the platform's latest header file
    // using bindgen have compatible signatures (or the same values if constants).
    // Since this is static assertion, we can detect problems with
    // `cargo check --tests --target <target>` run in CI (via TESTS=1 build.sh)
    // without actually running tests on these platforms.
    // See also tools/codegen/src/ffi.rs.
    // TODO(codegen): auto-generate this test
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::no_effect_underscore_binding
    )]
    const _: fn() = || {
        use test_helper::{libc, sys};
        let mut _sysctlbyname: unsafe extern "C" fn(
            *const ffi::c_char,
            *mut ffi::c_void,
            *mut ffi::c_size_t,
            *mut ffi::c_void,
            ffi::c_size_t,
        ) -> ffi::c_int = ffi::sysctlbyname;
        _sysctlbyname = libc::sysctlbyname;
        _sysctlbyname = sys::sysctlbyname;
    };
}
