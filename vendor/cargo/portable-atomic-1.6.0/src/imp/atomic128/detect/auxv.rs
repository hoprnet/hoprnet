// SPDX-License-Identifier: Apache-2.0 OR MIT

// Run-time feature detection on aarch64/powerpc64 Linux/Android/FreeBSD by parsing ELF auxiliary vectors.
//
// # Linux/Android
//
// As of nightly-2023-01-23, is_aarch64_feature_detected always uses dlsym by default
// on aarch64 Linux/Android, but on the following platforms, so we can safely assume
// getauxval is linked to the binary.
//
// - On glibc (*-linux-gnu*), [aarch64 support is available on glibc 2.17+](https://sourceware.org/legacy-ml/libc-announce/2012/msg00001.html)
//   and is newer than [glibc 2.16 that added getauxval](https://sourceware.org/legacy-ml/libc-announce/2012/msg00000.html).
// - On musl (*-linux-musl*, *-linux-ohos*), [aarch64 support is available on musl 1.1.7+](https://git.musl-libc.org/cgit/musl/tree/WHATSNEW?h=v1.1.7#n1422)
//   and is newer than [musl 1.1.0 that added getauxval](https://git.musl-libc.org/cgit/musl/tree/WHATSNEW?h=v1.1.0#n1197).
//   https://github.com/rust-lang/rust/commit/9a04ae4997493e9260352064163285cddc43de3c
// - On bionic (*-android*), [64-bit architecture support is available on Android 5.0+ (API level 21+)](https://android-developers.googleblog.com/2014/10/whats-new-in-android-50-lollipop.html)
//   and is newer than [Android 4.3 (API level 18) that added getauxval](https://github.com/aosp-mirror/platform_bionic/blob/d3ebc2f7c49a9893b114124d4a6b315f3a328764/libc/include/sys/auxv.h#L49).
//
// However, on musl with static linking, it seems that getauxval is not always available, independent of version requirements: https://github.com/rust-lang/rust/issues/89626
// (That problem may have been fixed in https://github.com/rust-lang/rust/commit/9a04ae4997493e9260352064163285cddc43de3c,
// but even in the version containing that patch, [there is report](https://github.com/rust-lang/rust/issues/89626#issuecomment-1242636038)
// of the same error.)
//
// On other Linux targets, we cannot assume that getauxval is always available, so we don't enable
// outline-atomics by default (can be enabled by `--cfg portable_atomic_outline_atomics`).
//
// - On musl with static linking. See the above for more.
//   Also, in this case, dlsym(getauxval) always returns null.
// - On uClibc-ng (*-linux-uclibc*, *-l4re-uclibc*), [uClibc-ng 1.0.43 (released in 2023-04-05) added getauxval](https://github.com/wbx-github/uclibc-ng/commit/d869bb1600942c01a77539128f9ba5b5b55ad647).
// - On Picolibc, [Picolibc 1.4.6 added getauxval stub](https://github.com/picolibc/picolibc#picolibc-version-146).
//
// See also https://github.com/rust-lang/stdarch/pull/1375
//
// See tests::test_linux_like and aarch64_aa64reg.rs for (test-only) alternative implementations.
//
// # FreeBSD
//
// As of nightly-2023-01-23, is_aarch64_feature_detected always uses mrs on
// aarch64 FreeBSD. However, they do not work on FreeBSD 12 on QEMU (confirmed
// on FreeBSD 12.{2,3,4}), and we got SIGILL (worked on FreeBSD 13 and 14).
//
// So use elf_aux_info instead of mrs like compiler-rt does.
// https://man.freebsd.org/elf_aux_info(3)
// https://reviews.llvm.org/D109330
//
// elf_aux_info is available on FreeBSD 12.0+ and 11.4+:
// https://github.com/freebsd/freebsd-src/commit/0b08ae2120cdd08c20a2b806e2fcef4d0a36c470
// https://github.com/freebsd/freebsd-src/blob/release/11.4.0/sys/sys/auxv.h
// On FreeBSD, [aarch64 support is available on FreeBSD 11.0+](https://www.freebsd.org/releases/11.0R/relnotes/#hardware-arm),
// but FreeBSD 11 (11.4) was EoL on 2021-09-30, and FreeBSD 11.3 was EoL on 2020-09-30:
// https://www.freebsd.org/security/unsupported
// See also https://github.com/rust-lang/stdarch/pull/611#issuecomment-445464613
//
// See tests::test_freebsd and aarch64_aa64reg.rs for (test-only) alternative implementations.
//
// # PowerPC64
//
// On PowerPC64, outline-atomics is currently disabled by default mainly for
// compatibility with older versions of operating systems
// (can be enabled by `--cfg portable_atomic_outline_atomics`).

include!("common.rs");

use os::ffi;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod os {
    // core::ffi::c_* (except c_void) requires Rust 1.64, libc will soon require Rust 1.47
    #[cfg_attr(test, allow(dead_code))]
    pub(super) mod ffi {
        pub(crate) use super::super::c_types::c_ulong;
        #[cfg(all(target_arch = "aarch64", target_os = "android"))]
        pub(crate) use super::super::c_types::{c_char, c_int};

        extern "C" {
            // https://man7.org/linux/man-pages/man3/getauxval.3.html
            // https://github.com/bminor/glibc/blob/801af9fafd4689337ebf27260aa115335a0cb2bc/misc/sys/auxv.h
            // https://github.com/bminor/musl/blob/7d756e1c04de6eb3f2b3d3e1141a218bb329fcfb/include/sys/auxv.h
            // https://github.com/wbx-github/uclibc-ng/blob/cdb07d2cd52af39feb425e6d36c02b30916b9f0a/include/sys/auxv.h
            // https://github.com/aosp-mirror/platform_bionic/blob/d3ebc2f7c49a9893b114124d4a6b315f3a328764/libc/include/sys/auxv.h
            // https://github.com/picolibc/picolibc/blob/7a8a58aeaa5946cb662577a518051091b691af3a/newlib/libc/picolib/getauxval.c
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/linux_like/linux/gnu/mod.rs#L1201
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/linux_like/linux/musl/mod.rs#L744
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/linux_like/android/b64/mod.rs#L333
            pub(crate) fn getauxval(type_: c_ulong) -> c_ulong;

            // Defined in sys/system_properties.h.
            // https://github.com/aosp-mirror/platform_bionic/blob/d3ebc2f7c49a9893b114124d4a6b315f3a328764/libc/include/sys/system_properties.h
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/linux_like/android/mod.rs#L3471
            #[cfg(all(target_arch = "aarch64", target_os = "android"))]
            pub(crate) fn __system_property_get(name: *const c_char, value: *mut c_char) -> c_int;
        }

        // https://github.com/torvalds/linux/blob/v6.1/include/uapi/linux/auxvec.h
        #[cfg(any(test, target_arch = "aarch64"))]
        pub(crate) const AT_HWCAP: c_ulong = 16;
        #[cfg(any(test, target_arch = "powerpc64"))]
        pub(crate) const AT_HWCAP2: c_ulong = 26;

        // Defined in sys/system_properties.h.
        // https://github.com/aosp-mirror/platform_bionic/blob/d3ebc2f7c49a9893b114124d4a6b315f3a328764/libc/include/sys/system_properties.h
        #[cfg(all(target_arch = "aarch64", target_os = "android"))]
        pub(crate) const PROP_VALUE_MAX: c_int = 92;
    }

    pub(super) fn getauxval(type_: ffi::c_ulong) -> ffi::c_ulong {
        #[cfg(all(target_arch = "aarch64", target_os = "android"))]
        {
            // Samsung Exynos 9810 has a bug that big and little cores have different
            // ISAs. And on older Android (pre-9), the kernel incorrectly reports
            // that features available only on some cores are available on all cores.
            // https://reviews.llvm.org/D114523
            let mut arch = [0_u8; ffi::PROP_VALUE_MAX as usize];
            // SAFETY: we've passed a valid C string and a buffer with max length.
            let len = unsafe {
                ffi::__system_property_get(
                    b"ro.arch\0".as_ptr().cast::<ffi::c_char>(),
                    arch.as_mut_ptr().cast::<ffi::c_char>(),
                )
            };
            // On Exynos, ro.arch is not available on Android 12+, but it is fine
            // because Android 9+ includes the fix.
            if len > 0 && arch.starts_with(b"exynos9810") {
                return 0;
            }
        }

        // SAFETY: `getauxval` is thread-safe. See also the module level docs.
        unsafe { ffi::getauxval(type_) }
    }
}
#[cfg(target_os = "freebsd")]
mod os {
    // core::ffi::c_* (except c_void) requires Rust 1.64, libc will soon require Rust 1.47
    #[cfg_attr(test, allow(dead_code))]
    pub(super) mod ffi {
        pub(crate) use super::super::c_types::{c_int, c_ulong, c_void};

        extern "C" {
            // Defined in sys/auxv.h.
            // https://man.freebsd.org/elf_aux_info(3)
            // https://github.com/freebsd/freebsd-src/blob/deb63adf945d446ed91a9d84124c71f15ae571d1/sys/sys/auxv.h
            pub(crate) fn elf_aux_info(aux: c_int, buf: *mut c_void, buf_len: c_int) -> c_int;
        }

        // Defined in sys/elf_common.h.
        // https://github.com/freebsd/freebsd-src/blob/deb63adf945d446ed91a9d84124c71f15ae571d1/sys/sys/elf_common.h
        #[cfg(any(test, target_arch = "aarch64"))]
        pub(crate) const AT_HWCAP: c_int = 25;
        #[cfg(any(test, target_arch = "powerpc64"))]
        pub(crate) const AT_HWCAP2: c_int = 26;
    }

    pub(super) fn getauxval(aux: ffi::c_int) -> ffi::c_ulong {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        const OUT_LEN: ffi::c_int = core::mem::size_of::<ffi::c_ulong>() as ffi::c_int;
        let mut out: ffi::c_ulong = 0;
        // SAFETY:
        // - the pointer is valid because we got it from a reference.
        // - `OUT_LEN` is the same as the size of `out`.
        // - `elf_aux_info` is thread-safe.
        unsafe {
            let res = ffi::elf_aux_info(
                aux,
                (&mut out as *mut ffi::c_ulong).cast::<ffi::c_void>(),
                OUT_LEN,
            );
            // If elf_aux_info fails, `out` will be left at zero (which is the proper default value).
            debug_assert!(res == 0 || out == 0);
        }
        out
    }
}

// Basically, Linux and FreeBSD use the same hwcap values.
// FreeBSD supports a subset of the hwcap values supported by Linux.
use arch::_detect;
#[cfg(target_arch = "aarch64")]
mod arch {
    use super::{ffi, os, CpuInfo};

    // Linux
    // https://github.com/torvalds/linux/blob/1c41041124bd14dd6610da256a3da4e5b74ce6b1/arch/arm64/include/uapi/asm/hwcap.h
    // FreeBSD
    // Defined in machine/elf.h.
    // https://github.com/freebsd/freebsd-src/blob/deb63adf945d446ed91a9d84124c71f15ae571d1/sys/arm64/include/elf.h
    // available on FreeBSD 13.0+ and 12.2+
    // https://github.com/freebsd/freebsd-src/blob/release/13.0.0/sys/arm64/include/elf.h
    // https://github.com/freebsd/freebsd-src/blob/release/12.2.0/sys/arm64/include/elf.h
    pub(super) const HWCAP_ATOMICS: ffi::c_ulong = 1 << 8;
    pub(super) const HWCAP_USCAT: ffi::c_ulong = 1 << 25;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[cfg(target_pointer_width = "64")]
    #[cfg(test)]
    pub(super) const HWCAP2_LRCPC3: ffi::c_ulong = 1 << 46;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[cfg(target_pointer_width = "64")]
    #[cfg(test)]
    pub(super) const HWCAP2_LSE128: ffi::c_ulong = 1 << 47;

    #[cold]
    pub(super) fn _detect(info: &mut CpuInfo) {
        let hwcap = os::getauxval(ffi::AT_HWCAP);

        if hwcap & HWCAP_ATOMICS != 0 {
            info.set(CpuInfo::HAS_LSE);
        }
        if hwcap & HWCAP_USCAT != 0 {
            info.set(CpuInfo::HAS_LSE2);
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        #[cfg(target_pointer_width = "64")]
        #[cfg(test)]
        {
            let hwcap2 = os::getauxval(ffi::AT_HWCAP2);
            if hwcap2 & HWCAP2_LRCPC3 != 0 {
                info.set(CpuInfo::HAS_RCPC3);
            }
            if hwcap2 & HWCAP2_LSE128 != 0 {
                info.set(CpuInfo::HAS_LSE128);
            }
        }
    }
}
#[cfg(target_arch = "powerpc64")]
mod arch {
    use super::{ffi, os, CpuInfo};

    // Linux
    // https://github.com/torvalds/linux/blob/v6.1/arch/powerpc/include/uapi/asm/cputable.h
    // FreeBSD
    // Defined in machine/cpu.h.
    // https://github.com/freebsd/freebsd-src/blob/deb63adf945d446ed91a9d84124c71f15ae571d1/sys/powerpc/include/cpu.h
    // available on FreeBSD 11.0+
    // https://github.com/freebsd/freebsd-src/commit/b0bf7fcd298133457991b27625bbed766e612730
    pub(super) const PPC_FEATURE2_ARCH_2_07: ffi::c_ulong = 0x80000000;

    #[cold]
    pub(super) fn _detect(info: &mut CpuInfo) {
        let hwcap2 = os::getauxval(ffi::AT_HWCAP2);

        // power8
        if hwcap2 & PPC_FEATURE2_ARCH_2_07 != 0 {
            info.set(CpuInfo::HAS_QUADWORD_ATOMICS);
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

    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_linux_like() {
        use c_types::*;
        use core::{arch::asm, mem};
        use std::vec;
        use test_helper::{libc, sys};

        // Linux kernel 6.4 has added a way to read auxv without depending on either libc or mrs trap.
        // https://github.com/torvalds/linux/commit/ddc65971bb677aa9f6a4c21f76d3133e106f88eb
        //
        // This is currently used only for testing.
        fn getauxval_pr_get_auxv(type_: ffi::c_ulong) -> Result<ffi::c_ulong, c_int> {
            #[cfg(target_arch = "aarch64")]
            unsafe fn prctl_get_auxv(out: *mut c_void, len: usize) -> Result<usize, c_int> {
                let r: i64;
                unsafe {
                    asm!(
                        "svc 0",
                        in("x8") sys::__NR_prctl as u64,
                        inout("x0") sys::PR_GET_AUXV as u64 => r,
                        in("x1") ptr_reg!(out),
                        in("x2") len as u64,
                        // arg4 and arg5 must be zero.
                        in("x3") 0_u64,
                        in("x4") 0_u64,
                        options(nostack, preserves_flags)
                    );
                }
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                if (r as c_int) < 0 {
                    Err(r as c_int)
                } else {
                    Ok(r as usize)
                }
            }
            #[cfg(target_arch = "powerpc64")]
            unsafe fn prctl_get_auxv(out: *mut c_void, len: usize) -> Result<usize, c_int> {
                let r: i64;
                unsafe {
                    asm!(
                        "sc",
                        "bns+ 2f",
                        "neg %r3, %r3",
                        "2:",
                        inout("r0") sys::__NR_prctl as u64 => _,
                        inout("r3") sys::PR_GET_AUXV as u64 => r,
                        inout("r4") ptr_reg!(out) => _,
                        inout("r5") len as u64 => _,
                        // arg4 and arg5 must be zero.
                        inout("r6") 0_u64 => _,
                        inout("r7") 0_u64 => _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") _,
                        out("cr0") _,
                        options(nostack, preserves_flags)
                    );
                }
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                if (r as c_int) < 0 {
                    Err(r as c_int)
                } else {
                    Ok(r as usize)
                }
            }

            let mut auxv = vec![unsafe { mem::zeroed::<sys::Elf64_auxv_t>() }; 38];

            let old_len = auxv.len() * mem::size_of::<sys::Elf64_auxv_t>();

            // SAFETY:
            // - `out_len` does not exceed the size of `auxv`.
            let _len = unsafe { prctl_get_auxv(auxv.as_mut_ptr().cast::<c_void>(), old_len)? };

            for aux in &auxv {
                if aux.a_type == type_ {
                    // SAFETY: aux.a_un is #[repr(C)] union and all fields have
                    // the same size and can be safely transmuted to integers.
                    return Ok(unsafe { aux.a_un.a_val });
                }
            }
            Err(0)
        }

        unsafe {
            let mut u = mem::zeroed();
            assert_eq!(libc::uname(&mut u), 0);
            let release = std::ffi::CStr::from_ptr(u.release.as_ptr());
            let release = core::str::from_utf8(release.to_bytes()).unwrap();
            let mut digits = release.split('.');
            let major = digits.next().unwrap().parse::<u32>().unwrap();
            let minor = digits.next().unwrap().parse::<u32>().unwrap();
            if (major, minor) < (6, 4) {
                std::eprintln!("kernel version: {major}.{minor} (no pr_get_auxv)");
                assert_eq!(getauxval_pr_get_auxv(ffi::AT_HWCAP).unwrap_err(), -22);
                assert_eq!(getauxval_pr_get_auxv(ffi::AT_HWCAP2).unwrap_err(), -22);
            } else {
                std::eprintln!("kernel version: {major}.{minor} (has pr_get_auxv)");
                assert_eq!(
                    os::getauxval(ffi::AT_HWCAP),
                    getauxval_pr_get_auxv(ffi::AT_HWCAP).unwrap()
                );
                assert_eq!(
                    os::getauxval(ffi::AT_HWCAP2),
                    getauxval_pr_get_auxv(ffi::AT_HWCAP2).unwrap()
                );
            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[cfg(all(target_arch = "aarch64", target_os = "android"))]
    #[test]
    fn test_android() {
        unsafe {
            let mut arch = [1; ffi::PROP_VALUE_MAX as usize];
            let len = ffi::__system_property_get(
                b"ro.arch\0".as_ptr().cast::<ffi::c_char>(),
                arch.as_mut_ptr().cast::<ffi::c_char>(),
            );
            assert!(len >= 0);
            std::eprintln!("len={}", len);
            std::eprintln!("arch={:?}", arch);
            std::eprintln!(
                "arch={:?}",
                core::str::from_utf8(core::slice::from_raw_parts(arch.as_ptr(), len as usize))
                    .unwrap()
            );
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    #[cfg(target_os = "freebsd")]
    #[test]
    fn test_freebsd() {
        use c_types::*;
        use core::{arch::asm, mem, ptr};
        use test_helper::sys;

        // This is almost equivalent to what elf_aux_info does.
        // https://man.freebsd.org/elf_aux_info(3)
        // On FreeBSD, [aarch64 support is available on FreeBSD 11.0+](https://www.freebsd.org/releases/11.0R/relnotes/#hardware-arm),
        // but elf_aux_info is available on FreeBSD 12.0+ and 11.4+:
        // https://github.com/freebsd/freebsd-src/commit/0b08ae2120cdd08c20a2b806e2fcef4d0a36c470
        // https://github.com/freebsd/freebsd-src/blob/release/11.4.0/sys/sys/auxv.h
        // so use sysctl instead of elf_aux_info.
        // Note that FreeBSD 11 (11.4) was EoL on 2021-09-30, and FreeBSD 11.3 was EoL on 2020-09-30:
        // https://www.freebsd.org/security/unsupported
        //
        // std_detect uses this way, but it appears to be somewhat incorrect
        // (the type of arg4 of sysctl, auxv is smaller than AT_COUNT, etc.).
        // https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/std_detect/src/detect/os/freebsd/auxvec.rs#L52
        //
        // This is currently used only for testing.
        // If you want us to use this implementation for compatibility with the older FreeBSD
        // version that came to EoL a few years ago, please open an issue.
        fn getauxval_sysctl_libc(type_: ffi::c_int) -> ffi::c_ulong {
            let mut auxv: [sys::Elf64_Auxinfo; sys::AT_COUNT as usize] = unsafe { mem::zeroed() };

            let mut len = core::mem::size_of_val(&auxv) as c_size_t;

            // SAFETY: calling getpid is safe.
            let pid = unsafe { sys::getpid() };
            let mib = [
                sys::CTL_KERN as c_int,
                sys::KERN_PROC as c_int,
                sys::KERN_PROC_AUXV as c_int,
                pid,
            ];

            #[allow(clippy::cast_possible_truncation)]
            // SAFETY:
            // - `mib.len()` does not exceed the size of `mib`.
            // - `len` does not exceed the size of `auxv`.
            // - `sysctl` is thread-safe.
            let res = unsafe {
                sys::sysctl(
                    mib.as_ptr(),
                    mib.len() as c_uint,
                    auxv.as_mut_ptr().cast::<c_void>(),
                    &mut len,
                    ptr::null_mut(),
                    0,
                )
            };

            if res != -1 {
                for aux in &auxv {
                    if aux.a_type == type_ as c_long {
                        // SAFETY: aux.a_un is #[repr(C)] union and all fields have
                        // the same size and can be safely transmuted to integers.
                        return unsafe { aux.a_un.a_val as c_ulong };
                    }
                }
            }
            0
        }
        // Similar to the above, but call syscall using asm instead of libc.
        // Note that FreeBSD does not guarantee the stability of raw syscall as
        // much as Linux does (It may actually be stable enough, though:
        // https://lists.llvm.org/pipermail/llvm-dev/2019-June/133393.html,
        // https://github.com/ziglang/zig/issues/16590).
        //
        // This is currently used only for testing.
        fn getauxval_sysctl_asm_syscall(type_: ffi::c_int) -> Result<ffi::c_ulong, c_int> {
            #[allow(non_camel_case_types)]
            type pid_t = c_int;

            // https://github.com/freebsd/freebsd-src/blob/9888a79adad22ba06b5aff17d05abac0029c537a/lib/libc/aarch64/SYS.h
            // https://github.com/golang/go/blob/4badad8d477ffd7a6b762c35bc69aed82faface7/src/syscall/asm_freebsd_arm64.s
            #[cfg(target_arch = "aarch64")]
            #[inline]
            fn getpid() -> pid_t {
                #[allow(clippy::cast_possible_truncation)]
                // SAFETY: calling getpid is safe.
                unsafe {
                    let n = sys::SYS_getpid;
                    let r: i64;
                    asm!(
                        "svc 0",
                        in("x8") n as u64,
                        out("x0") r,
                        options(nostack, readonly),
                    );
                    r as pid_t
                }
            }
            #[cfg(target_arch = "aarch64")]
            #[inline]
            unsafe fn sysctl(
                name: *const c_int,
                name_len: c_uint,
                old_p: *mut c_void,
                old_len_p: *mut c_size_t,
                new_p: *const c_void,
                new_len: c_size_t,
            ) -> Result<c_int, c_int> {
                #[allow(clippy::cast_possible_truncation)]
                // SAFETY: the caller must uphold the safety contract.
                unsafe {
                    let mut n = sys::SYS___sysctl as u64;
                    let r: i64;
                    asm!(
                        "svc 0",
                        "b.cc 2f",
                        "mov x8, x0",
                        "mov x0, #-1",
                        "2:",
                        inout("x8") n,
                        inout("x0") ptr_reg!(name) => r,
                        inout("x1") name_len as u64 => _,
                        in("x2") ptr_reg!(old_p),
                        in("x3") ptr_reg!(old_len_p),
                        in("x4") ptr_reg!(new_p),
                        in("x5") new_len as u64,
                        options(nostack),
                    );
                    if r as c_int == -1 {
                        Err(n as c_int)
                    } else {
                        Ok(r as c_int)
                    }
                }
            }

            // https://github.com/freebsd/freebsd-src/blob/9888a79adad22ba06b5aff17d05abac0029c537a/lib/libc/powerpc64/SYS.h
            #[cfg(target_arch = "powerpc64")]
            #[inline]
            fn getpid() -> pid_t {
                #[allow(clippy::cast_possible_truncation)]
                // SAFETY: calling getpid is safe.
                unsafe {
                    let n = sys::SYS_getpid;
                    let r: i64;
                    asm!(
                        "sc",
                        inout("r0") n as u64 => _,
                        out("r3") r,
                        out("r4") _,
                        out("r5") _,
                        out("r6") _,
                        out("r7") _,
                        out("r8") _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") _,
                        out("cr0") _,
                        options(nostack, preserves_flags, readonly),
                    );
                    r as pid_t
                }
            }
            #[cfg(target_arch = "powerpc64")]
            #[inline]
            unsafe fn sysctl(
                name: *const c_int,
                name_len: c_uint,
                old_p: *mut c_void,
                old_len_p: *mut c_size_t,
                new_p: *const c_void,
                new_len: c_size_t,
            ) -> Result<c_int, c_int> {
                #[allow(clippy::cast_possible_truncation)]
                // SAFETY: the caller must uphold the safety contract.
                unsafe {
                    let mut n = sys::SYS___sysctl as u64;
                    let r: i64;
                    asm!(
                        "sc",
                        "bns+ 2f",
                        "mr %r0, %r3",
                        "li %r3, -1",
                        "2:",
                        inout("r0") n,
                        inout("r3") ptr_reg!(name) => r,
                        inout("r4") name_len as u64 => _,
                        inout("r5") ptr_reg!(old_p) => _,
                        inout("r6") ptr_reg!(old_len_p) => _,
                        inout("r7") ptr_reg!(new_p) => _,
                        inout("r8") new_len as u64 => _,
                        out("r9") _,
                        out("r10") _,
                        out("r11") _,
                        out("r12") _,
                        out("cr0") _,
                        options(nostack, preserves_flags)
                    );
                    if r as c_int == -1 {
                        Err(n as c_int)
                    } else {
                        Ok(r as c_int)
                    }
                }
            }

            let mut auxv: [sys::Elf64_Auxinfo; sys::AT_COUNT as usize] = unsafe { mem::zeroed() };

            let mut len = core::mem::size_of_val(&auxv) as c_size_t;

            let pid = getpid();
            let mib = [
                sys::CTL_KERN as c_int,
                sys::KERN_PROC as c_int,
                sys::KERN_PROC_AUXV as c_int,
                pid,
            ];

            #[allow(clippy::cast_possible_truncation)]
            // SAFETY:
            // - `mib.len()` does not exceed the size of `mib`.
            // - `len` does not exceed the size of `auxv`.
            // - `sysctl` is thread-safe.
            unsafe {
                sysctl(
                    mib.as_ptr(),
                    mib.len() as c_uint,
                    auxv.as_mut_ptr().cast::<c_void>(),
                    &mut len,
                    ptr::null_mut(),
                    0,
                )?;
            }

            for aux in &auxv {
                if aux.a_type == type_ as c_long {
                    // SAFETY: aux.a_un is #[repr(C)] union and all fields have
                    // the same size and can be safely transmuted to integers.
                    return Ok(unsafe { aux.a_un.a_val as c_ulong });
                }
            }
            Err(0)
        }

        assert_eq!(os::getauxval(ffi::AT_HWCAP), getauxval_sysctl_libc(ffi::AT_HWCAP));
        assert_eq!(os::getauxval(ffi::AT_HWCAP2), getauxval_sysctl_libc(ffi::AT_HWCAP2));
        assert_eq!(
            os::getauxval(ffi::AT_HWCAP),
            getauxval_sysctl_asm_syscall(ffi::AT_HWCAP).unwrap()
        );
        assert_eq!(
            os::getauxval(ffi::AT_HWCAP2),
            // AT_HWCAP2 is only available on FreeBSD 13+, at least for aarch64.
            getauxval_sysctl_asm_syscall(ffi::AT_HWCAP2).unwrap_or(0)
        );
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
        clippy::cast_possible_truncation,
        clippy::no_effect_underscore_binding
    )]
    const _: fn() = || {
        use test_helper::{libc, sys};
        #[cfg(not(target_os = "freebsd"))]
        type AtType = ffi::c_ulong;
        #[cfg(target_os = "freebsd")]
        type AtType = ffi::c_int;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let mut _getauxval: unsafe extern "C" fn(ffi::c_ulong) -> ffi::c_ulong = ffi::getauxval;
            _getauxval = libc::getauxval;
            _getauxval = sys::getauxval;
        }
        #[cfg(all(target_arch = "aarch64", target_os = "android"))]
        {
            let mut ___system_property_get: unsafe extern "C" fn(
                *const ffi::c_char,
                *mut ffi::c_char,
            ) -> ffi::c_int = ffi::__system_property_get;
            ___system_property_get = libc::__system_property_get;
            ___system_property_get = sys::__system_property_get;
            static_assert!(ffi::PROP_VALUE_MAX == libc::PROP_VALUE_MAX);
            static_assert!(ffi::PROP_VALUE_MAX == sys::PROP_VALUE_MAX as ffi::c_int);
        }
        #[cfg(target_os = "freebsd")]
        {
            let mut _elf_aux_info: unsafe extern "C" fn(
                ffi::c_int,
                *mut ffi::c_void,
                ffi::c_int,
            ) -> ffi::c_int = ffi::elf_aux_info;
            _elf_aux_info = libc::elf_aux_info;
            _elf_aux_info = sys::elf_aux_info;
        }
        #[cfg(not(target_os = "freebsd"))] // libc doesn't have this on FreeBSD
        static_assert!(ffi::AT_HWCAP == libc::AT_HWCAP);
        static_assert!(ffi::AT_HWCAP == sys::AT_HWCAP as AtType);
        #[cfg(not(target_os = "freebsd"))] // libc doesn't have this on FreeBSD
        static_assert!(ffi::AT_HWCAP2 == libc::AT_HWCAP2);
        static_assert!(ffi::AT_HWCAP2 == sys::AT_HWCAP2 as AtType);
        #[cfg(target_arch = "aarch64")]
        {
            // static_assert!(arch::HWCAP_ATOMICS == libc::HWCAP_ATOMICS); // libc doesn't have this
            static_assert!(arch::HWCAP_ATOMICS == sys::HWCAP_ATOMICS as ffi::c_ulong);
            // static_assert!(HWCAP_USCAT == libc::HWCAP_USCAT); // libc doesn't have this
            static_assert!(arch::HWCAP_USCAT == sys::HWCAP_USCAT as ffi::c_ulong);
            #[cfg(any(target_os = "linux", target_os = "android"))]
            #[cfg(target_pointer_width = "64")]
            {
                // static_assert!(HWCAP2_LRCPC3 == libc::HWCAP2_LRCPC3); // libc doesn't have this
                static_assert!(arch::HWCAP2_LRCPC3 == sys::HWCAP2_LRCPC3 as ffi::c_ulong);
                // static_assert!(HWCAP2_LSE128 == libc::HWCAP2_LSE128); // libc doesn't have this
                static_assert!(arch::HWCAP2_LSE128 == sys::HWCAP2_LSE128 as ffi::c_ulong);
            }
        }
        #[cfg(target_arch = "powerpc64")]
        {
            // static_assert!(arch::PPC_FEATURE2_ARCH_2_07 == libc::PPC_FEATURE2_ARCH_2_07); // libc doesn't have this
            static_assert!(
                arch::PPC_FEATURE2_ARCH_2_07 == sys::PPC_FEATURE2_ARCH_2_07 as ffi::c_ulong
            );
        }
    };
}
