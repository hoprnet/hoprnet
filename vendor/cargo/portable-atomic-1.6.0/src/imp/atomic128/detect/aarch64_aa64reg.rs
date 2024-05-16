// SPDX-License-Identifier: Apache-2.0 OR MIT

// Run-time feature detection on aarch64 Linux/FreeBSD/NetBSD/OpenBSD by parsing system registers.
//
// As of nightly-2023-01-23, is_aarch64_feature_detected doesn't support run-time detection on NetBSD/OpenBSD.
// https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/std_detect/src/detect/mod.rs
// https://github.com/rust-lang/stdarch/pull/1374
//
// Refs:
// - https://developer.arm.com/documentation/ddi0601/latest/AArch64-Registers
// - https://www.kernel.org/doc/Documentation/arm64/cpu-feature-registers.txt
// - https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/std_detect/src/detect/os/aarch64.rs
//
// Supported platforms:
// - Linux 4.11+ (emulate mrs instruction)
//   https://github.com/torvalds/linux/commit/77c97b4ee21290f5f083173d957843b615abbff2
// - FreeBSD 12.0+ (emulate mrs instruction)
//   https://github.com/freebsd/freebsd-src/commit/398810619cb32abf349f8de23f29510b2ee0839b
// - NetBSD 9.0+ (through sysctl)
//   https://github.com/NetBSD/src/commit/0e9d25528729f7fea53e78275d1bc5039dfe8ffb
// - OpenBSD 7.1+ (through sysctl)
//   https://github.com/openbsd/src/commit/d335af936b9d7dd9cf655cae1ce19560c45de6c8
//
// For now, this module is only used on NetBSD/OpenBSD.
// On Linux/FreeBSD, this module is test-only:
// - On Linux, this approach requires a higher kernel version than Rust supports,
//   and also does not work with qemu-user (as of QEMU 7.2) and Valgrind.
//   (Looking into HWCAP_CPUID in auxvec, it appears that Valgrind is setting it
//   to false correctly, but qemu-user is setting it to true.)
// - On FreeBSD, this approach does not work on FreeBSD 12 on QEMU (confirmed on
//   FreeBSD 12.{2,3,4}), and we got SIGILL (worked on FreeBSD 13 and 14).

include!("common.rs");

#[cfg_attr(test, derive(Debug, PartialEq))]
struct AA64Reg {
    aa64isar0: u64,
    #[cfg(test)]
    aa64isar1: u64,
    // OpenBSD has an API to get this, but currently always returns 0.
    // https://github.com/openbsd/src/blob/6a233889798dc3ecb18acc52dce1e57862af2957/sys/arch/arm64/arm64/machdep.c#L371-L377
    #[cfg_attr(target_os = "openbsd", cfg(test))]
    aa64mmfr2: u64,
}

#[cold]
fn _detect(info: &mut CpuInfo) {
    let AA64Reg {
        aa64isar0,
        #[cfg(test)]
        aa64isar1,
        #[cfg_attr(target_os = "openbsd", cfg(test))]
        aa64mmfr2,
    } = imp::aa64reg();

    // ID_AA64ISAR0_EL1, Instruction Set Attribute Register 0
    // https://developer.arm.com/documentation/ddi0601/2023-06/AArch64-Registers/ID-AA64ISAR0-EL1--AArch64-Instruction-Set-Attribute-Register-0?lang=en
    let atomic = extract(aa64isar0, 23, 20);
    if atomic >= 2 {
        info.set(CpuInfo::HAS_LSE);
        // we currently only use FEAT_LSE and FEAT_LSE2 in outline-atomics.
        #[cfg(test)]
        {
            if atomic >= 3 {
                info.set(CpuInfo::HAS_LSE128);
            }
        }
    }
    // we currently only use FEAT_LSE and FEAT_LSE2 in outline-atomics.
    #[cfg(test)]
    {
        // ID_AA64ISAR1_EL1, Instruction Set Attribute Register 1
        // https://developer.arm.com/documentation/ddi0601/2023-06/AArch64-Registers/ID-AA64ISAR1-EL1--AArch64-Instruction-Set-Attribute-Register-1?lang=en
        if extract(aa64isar1, 23, 20) >= 3 {
            info.set(CpuInfo::HAS_RCPC3);
        }
    }
    // OpenBSD has an API to get this, but currently always returns 0.
    // https://github.com/openbsd/src/blob/6a233889798dc3ecb18acc52dce1e57862af2957/sys/arch/arm64/arm64/machdep.c#L371-L377
    #[cfg_attr(target_os = "openbsd", cfg(test))]
    {
        // ID_AA64MMFR2_EL1, AArch64 Memory Model Feature Register 2
        // https://developer.arm.com/documentation/ddi0601/2023-06/AArch64-Registers/ID-AA64MMFR2-EL1--AArch64-Memory-Model-Feature-Register-2?lang=en
        if extract(aa64mmfr2, 35, 32) >= 1 {
            info.set(CpuInfo::HAS_LSE2);
        }
    }
}

fn extract(x: u64, high: usize, low: usize) -> u64 {
    (x >> low) & ((1 << (high - low + 1)) - 1)
}

#[cfg(not(any(target_os = "netbsd", target_os = "openbsd")))]
mod imp {
    // This module is test-only. See parent module docs for details.

    #[cfg(not(portable_atomic_no_asm))]
    use core::arch::asm;

    use super::AA64Reg;

    pub(super) fn aa64reg() -> AA64Reg {
        // SAFETY: This is safe on FreeBSD 12.0+. FreeBSD 11 was EoL on 2021-09-30.
        // Note that stdarch has been doing the same thing since before FreeBSD 11 was EoL.
        // https://github.com/rust-lang/stdarch/pull/611
        unsafe {
            let aa64isar0: u64;
            asm!(
                "mrs {0}, ID_AA64ISAR0_EL1",
                out(reg) aa64isar0,
                options(pure, nomem, nostack, preserves_flags)
            );
            #[cfg(test)]
            let aa64isar1: u64;
            #[cfg(test)]
            {
                asm!(
                    "mrs {0}, ID_AA64ISAR1_EL1",
                    out(reg) aa64isar1,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
            let aa64mmfr2: u64;
            asm!(
                "mrs {0}, ID_AA64MMFR2_EL1",
                out(reg) aa64mmfr2,
                options(pure, nomem, nostack, preserves_flags)
            );
            AA64Reg {
                aa64isar0,
                #[cfg(test)]
                aa64isar1,
                aa64mmfr2,
            }
        }
    }
}
#[cfg(target_os = "netbsd")]
mod imp {
    // NetBSD doesn't trap the mrs instruction, but exposes the system registers through sysctl.
    // https://github.com/NetBSD/src/commit/0e9d25528729f7fea53e78275d1bc5039dfe8ffb
    // https://github.com/golang/sys/commit/ef9fd89ba245e184bdd308f7f2b4f3c551fa5b0f

    use core::ptr;

    use super::AA64Reg;

    // core::ffi::c_* (except c_void) requires Rust 1.64, libc will soon require Rust 1.47
    #[allow(non_camel_case_types)]
    pub(super) mod ffi {
        pub(crate) use super::super::c_types::{c_char, c_int, c_size_t, c_void};

        extern "C" {
            // Defined in sys/sysctl.h.
            // https://man.netbsd.org/sysctl.3
            // https://github.com/NetBSD/src/blob/167403557cf60bed09a63fc84d941a1a4bd7d52e/sys/sys/sysctl.h
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/bsd/netbsdlike/netbsd/mod.rs#L2582
            pub(crate) fn sysctlbyname(
                name: *const c_char,
                old_p: *mut c_void,
                old_len_p: *mut c_size_t,
                new_p: *const c_void,
                new_len: c_size_t,
            ) -> c_int;
        }

        // Defined in aarch64/armreg.h.
        // https://github.com/NetBSD/src/blob/167403557cf60bed09a63fc84d941a1a4bd7d52e/sys/arch/aarch64/include/armreg.h#L1626
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub(crate) struct aarch64_sysctl_cpu_id {
            // NetBSD 9.0+
            // https://github.com/NetBSD/src/commit/0e9d25528729f7fea53e78275d1bc5039dfe8ffb
            pub(crate) midr: u64,
            pub(crate) revidr: u64,
            pub(crate) mpidr: u64,
            pub(crate) aa64dfr0: u64,
            pub(crate) aa64dfr1: u64,
            pub(crate) aa64isar0: u64,
            pub(crate) aa64isar1: u64,
            pub(crate) aa64mmfr0: u64,
            pub(crate) aa64mmfr1: u64,
            pub(crate) aa64mmfr2: u64,
            pub(crate) aa64pfr0: u64,
            pub(crate) aa64pfr1: u64,
            pub(crate) aa64zfr0: u64,
            pub(crate) mvfr0: u32,
            pub(crate) mvfr1: u32,
            pub(crate) mvfr2: u32,
            // NetBSD 10.0+
            // https://github.com/NetBSD/src/commit/0c7bdc13f0e332cccec56e307f023b4888638973
            pub(crate) pad: u32,
            pub(crate) clidr: u64,
            pub(crate) ctr: u64,
        }
    }

    pub(super) unsafe fn sysctl_cpu_id(name: &[u8]) -> Option<AA64Reg> {
        const OUT_LEN: ffi::c_size_t =
            core::mem::size_of::<ffi::aarch64_sysctl_cpu_id>() as ffi::c_size_t;

        debug_assert_eq!(name.last(), Some(&0), "{:?}", name);
        debug_assert_eq!(name.iter().filter(|&&v| v == 0).count(), 1, "{:?}", name);

        // SAFETY: all fields of aarch64_sysctl_cpu_id are zero-able and we use
        // the result when machdep.cpuN.cpu_id sysctl was successful.
        let mut buf: ffi::aarch64_sysctl_cpu_id = unsafe { core::mem::zeroed() };
        let mut out_len = OUT_LEN;
        // SAFETY:
        // - the caller must guarantee that `name` is ` machdep.cpuN.cpu_id` in a C string.
        // - `out_len` does not exceed the size of the value at `buf`.
        // - `sysctlbyname` is thread-safe.
        let res = unsafe {
            ffi::sysctlbyname(
                name.as_ptr().cast::<ffi::c_char>(),
                (&mut buf as *mut ffi::aarch64_sysctl_cpu_id).cast::<ffi::c_void>(),
                &mut out_len,
                ptr::null_mut(),
                0,
            )
        };
        if res != 0 {
            return None;
        }
        Some(AA64Reg {
            aa64isar0: buf.aa64isar0,
            #[cfg(test)]
            aa64isar1: buf.aa64isar1,
            aa64mmfr2: buf.aa64mmfr2,
        })
    }

    pub(super) fn aa64reg() -> AA64Reg {
        // Get system registers for cpu0.
        // If failed, returns default because machdep.cpuN.cpu_id sysctl is not available.
        // machdep.cpuN.cpu_id sysctl was added on NetBSD 9.0 so it is not available on older versions.
        // SAFETY: we passed a valid name in a C string.
        // It is ok to check only cpu0, even if there are more CPUs.
        // https://github.com/NetBSD/src/commit/bd9707e06ea7d21b5c24df6dfc14cb37c2819416
        // https://github.com/golang/sys/commit/ef9fd89ba245e184bdd308f7f2b4f3c551fa5b0f
        match unsafe { sysctl_cpu_id(b"machdep.cpu0.cpu_id\0") } {
            Some(cpu_id) => cpu_id,
            None => AA64Reg {
                aa64isar0: 0,
                #[cfg(test)]
                aa64isar1: 0,
                aa64mmfr2: 0,
            },
        }
    }
}
#[cfg(target_os = "openbsd")]
mod imp {
    // OpenBSD doesn't trap the mrs instruction, but exposes the system registers through sysctl.
    // https://github.com/openbsd/src/commit/d335af936b9d7dd9cf655cae1ce19560c45de6c8
    // https://github.com/golang/go/commit/cd54ef1f61945459486e9eea2f016d99ef1da925

    use core::ptr;

    use super::AA64Reg;

    // core::ffi::c_* (except c_void) requires Rust 1.64, libc will soon require Rust 1.47
    #[allow(non_camel_case_types)]
    pub(super) mod ffi {
        pub(crate) use super::super::c_types::{c_int, c_size_t, c_uint, c_void};

        // Defined in sys/sysctl.h.
        // https://github.com/openbsd/src/blob/72ccc03bd11da614f31f7ff76e3f6fce99bc1c79/sys/sys/sysctl.h#L82
        pub(crate) const CTL_MACHDEP: c_int = 7;
        // Defined in machine/cpu.h.
        // https://github.com/openbsd/src/blob/72ccc03bd11da614f31f7ff76e3f6fce99bc1c79/sys/arch/arm64/include/cpu.h#L25-L40
        pub(crate) const CPU_ID_AA64ISAR0: c_int = 2;
        #[cfg(test)]
        pub(crate) const CPU_ID_AA64ISAR1: c_int = 3;
        // OpenBSD has an API to get this, but currently always returns 0.
        // https://github.com/openbsd/src/blob/6a233889798dc3ecb18acc52dce1e57862af2957/sys/arch/arm64/arm64/machdep.c#L371-L377
        #[cfg(test)]
        pub(crate) const CPU_ID_AA64MMFR2: c_int = 7;

        extern "C" {
            // Defined in sys/sysctl.h.
            // https://man.openbsd.org/sysctl.2
            // https://github.com/openbsd/src/blob/72ccc03bd11da614f31f7ff76e3f6fce99bc1c79/sys/sys/sysctl.h
            // https://github.com/rust-lang/libc/blob/0.2.139/src/unix/bsd/netbsdlike/openbsd/mod.rs#L1817-L1824
            pub(crate) fn sysctl(
                name: *const c_int,
                name_len: c_uint,
                old_p: *mut c_void,
                old_len_p: *mut c_size_t,
                new_p: *mut c_void,
                new_len: c_size_t,
            ) -> c_int;
        }
    }

    // ID_AA64ISAR0_EL1 and ID_AA64ISAR1_EL1 are supported on OpenBSD 7.1+.
    // https://github.com/openbsd/src/commit/d335af936b9d7dd9cf655cae1ce19560c45de6c8
    // Others are supported on OpenBSD 7.3+.
    // https://github.com/openbsd/src/commit/c7654cd65262d532212f65123ee3905ba200365c
    // sysctl returns an unsupported error if operation is not supported,
    // so we can safely use this function on older versions of OpenBSD.
    pub(super) fn aa64reg() -> AA64Reg {
        let aa64isar0 = sysctl64(&[ffi::CTL_MACHDEP, ffi::CPU_ID_AA64ISAR0]).unwrap_or(0);
        #[cfg(test)]
        let aa64isar1 = sysctl64(&[ffi::CTL_MACHDEP, ffi::CPU_ID_AA64ISAR1]).unwrap_or(0);
        #[cfg(test)]
        let aa64mmfr2 = sysctl64(&[ffi::CTL_MACHDEP, ffi::CPU_ID_AA64MMFR2]).unwrap_or(0);
        AA64Reg {
            aa64isar0,
            #[cfg(test)]
            aa64isar1,
            #[cfg(test)]
            aa64mmfr2,
        }
    }

    fn sysctl64(mib: &[ffi::c_int]) -> Option<u64> {
        const OUT_LEN: ffi::c_size_t = core::mem::size_of::<u64>() as ffi::c_size_t;
        let mut out = 0_u64;
        let mut out_len = OUT_LEN;
        #[allow(clippy::cast_possible_truncation)]
        // SAFETY:
        // - `mib.len()` does not exceed the size of `mib`.
        // - `out_len` does not exceed the size of `out`.
        // - `sysctl` is thread-safe.
        let res = unsafe {
            ffi::sysctl(
                mib.as_ptr(),
                mib.len() as ffi::c_uint,
                (&mut out as *mut u64).cast::<ffi::c_void>(),
                &mut out_len,
                ptr::null_mut(),
                0,
            )
        };
        if res == -1 {
            return None;
        }
        debug_assert_eq!(out_len, OUT_LEN);
        Some(out)
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
    use std::{
        process::Command,
        string::{String, ToString},
    };

    use super::*;

    #[test]
    fn test_aa64reg() {
        let AA64Reg { aa64isar0, aa64isar1, aa64mmfr2 } = imp::aa64reg();
        std::eprintln!("aa64isar0={}", aa64isar0);
        std::eprintln!("aa64isar1={}", aa64isar1);
        std::eprintln!("aa64mmfr2={}", aa64mmfr2);
        if cfg!(target_os = "openbsd") {
            let output = Command::new("sysctl").arg("machdep").output().unwrap();
            assert!(output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();
            // OpenBSD 7.1+
            assert_eq!(
                stdout.lines().find_map(|s| s.strip_prefix("machdep.id_aa64isar0=")).unwrap_or("0"),
                aa64isar0.to_string(),
            );
            assert_eq!(
                stdout.lines().find_map(|s| s.strip_prefix("machdep.id_aa64isar1=")).unwrap_or("0"),
                aa64isar1.to_string(),
            );
            // OpenBSD 7.3+
            assert_eq!(
                stdout.lines().find_map(|s| s.strip_prefix("machdep.id_aa64mmfr2=")).unwrap_or("0"),
                aa64mmfr2.to_string(),
            );
        }
        if detect().test(CpuInfo::HAS_LSE) {
            let atomic = extract(aa64isar0, 23, 20);
            if detect().test(CpuInfo::HAS_LSE128) {
                assert_eq!(atomic, 3);
            } else {
                assert_eq!(atomic, 2);
            }
        }
        if detect().test(CpuInfo::HAS_LSE2) {
            assert_eq!(extract(aa64mmfr2, 35, 32), 1);
        }
        if detect().test(CpuInfo::HAS_RCPC3) {
            assert_eq!(extract(aa64isar1, 23, 20), 3);
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    #[cfg(target_os = "netbsd")]
    #[test]
    fn test_netbsd() {
        use c_types::*;
        use core::{arch::asm, mem, ptr};
        use imp::ffi;
        use test_helper::sys;

        // Call syscall using asm instead of libc.
        // Note that NetBSD does not guarantee the stability of raw syscall as
        // much as Linux does (It may actually be stable enough, though: https://lists.llvm.org/pipermail/llvm-dev/2019-June/133393.html).
        //
        // This is currently used only for testing.
        unsafe fn sysctl_cpu_id_asm_syscall(name: &[&[u8]]) -> Result<AA64Reg, c_int> {
            // https://github.com/golang/go/blob/4badad8d477ffd7a6b762c35bc69aed82faface7/src/syscall/asm_netbsd_arm64.s
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
                        "mov x17, x0",
                        "mov x0, #-1",
                        "2:",
                        inout("x17") n,
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

            // https://github.com/golang/sys/blob/4badad8d477ffd7a6b762c35bc69aed82faface7/cpu/cpu_netbsd_arm64.go.
            use std::{vec, vec::Vec};
            fn sysctl_nodes(mib: &mut Vec<i32>) -> Result<Vec<sys::sysctlnode>, i32> {
                mib.push(sys::CTL_QUERY);
                let mut q_node = sys::sysctlnode {
                    sysctl_flags: sys::SYSCTL_VERS_1,
                    ..unsafe { mem::zeroed() }
                };
                let qp = (&mut q_node as *mut sys::sysctlnode).cast::<ffi::c_void>();
                let sz = mem::size_of::<sys::sysctlnode>();
                let mut olen = 0;
                #[allow(clippy::cast_possible_truncation)]
                unsafe {
                    sysctl(mib.as_ptr(), mib.len() as c_uint, ptr::null_mut(), &mut olen, qp, sz)?;
                }

                let mut nodes = Vec::<sys::sysctlnode>::with_capacity(olen / sz);
                let np = nodes.as_mut_ptr().cast::<ffi::c_void>();
                #[allow(clippy::cast_possible_truncation)]
                unsafe {
                    sysctl(mib.as_ptr(), mib.len() as c_uint, np, &mut olen, qp, sz)?;
                    nodes.set_len(olen / sz);
                }

                mib.pop(); // pop CTL_QUERY
                Ok(nodes)
            }
            fn name_to_mib(parts: &[&[u8]]) -> Result<Vec<i32>, i32> {
                let mut mib = vec![];
                for (part_no, &part) in parts.iter().enumerate() {
                    let nodes = sysctl_nodes(&mut mib)?;
                    for node in nodes {
                        let mut n = vec![];
                        for b in node.sysctl_name {
                            if b != 0 {
                                n.push(b);
                            }
                        }
                        if n == part {
                            mib.push(node.sysctl_num);
                            break;
                        }
                    }
                    if mib.len() != part_no + 1 {
                        return Err(0);
                    }
                }

                Ok(mib)
            }

            const OUT_LEN: ffi::c_size_t =
                core::mem::size_of::<ffi::aarch64_sysctl_cpu_id>() as ffi::c_size_t;

            let mib = name_to_mib(name)?;

            let mut buf: ffi::aarch64_sysctl_cpu_id = unsafe { core::mem::zeroed() };
            let mut out_len = OUT_LEN;
            #[allow(clippy::cast_possible_truncation)]
            unsafe {
                sysctl(
                    mib.as_ptr(),
                    mib.len() as c_uint,
                    (&mut buf as *mut ffi::aarch64_sysctl_cpu_id).cast::<ffi::c_void>(),
                    &mut out_len,
                    ptr::null_mut(),
                    0,
                )?;
            }
            Ok(AA64Reg {
                aa64isar0: buf.aa64isar0,
                #[cfg(test)]
                aa64isar1: buf.aa64isar1,
                #[cfg(test)]
                aa64mmfr2: buf.aa64mmfr2,
            })
        }

        unsafe {
            assert_eq!(
                imp::sysctl_cpu_id(b"machdep.cpu0.cpu_id\0").unwrap(),
                sysctl_cpu_id_asm_syscall(&[b"machdep", b"cpu0", b"cpu_id"]).unwrap()
            );
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
    #[cfg(target_os = "netbsd")]
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::no_effect_underscore_binding,
        clippy::used_underscore_binding
    )]
    const _: fn() = || {
        use core::mem::size_of;
        use imp::ffi;
        use test_helper::{libc, sys};
        let mut _sysctlbyname: unsafe extern "C" fn(
            *const ffi::c_char,
            *mut ffi::c_void,
            *mut ffi::c_size_t,
            *const ffi::c_void,
            ffi::c_size_t,
        ) -> ffi::c_int = ffi::sysctlbyname;
        _sysctlbyname = libc::sysctlbyname;
        _sysctlbyname = sys::sysctlbyname;
        // libc doesn't have this
        // static_assert!(
        //     size_of::<ffi::aarch64_sysctl_cpu_id>() == size_of::<libc::aarch64_sysctl_cpu_id>()
        // );
        static_assert!(
            size_of::<ffi::aarch64_sysctl_cpu_id>() == size_of::<sys::aarch64_sysctl_cpu_id>()
        );
        let ffi: ffi::aarch64_sysctl_cpu_id = unsafe { core::mem::zeroed() };
        let _ = sys::aarch64_sysctl_cpu_id {
            ac_midr: ffi.midr,
            ac_revidr: ffi.revidr,
            ac_mpidr: ffi.mpidr,
            ac_aa64dfr0: ffi.aa64dfr0,
            ac_aa64dfr1: ffi.aa64dfr1,
            ac_aa64isar0: ffi.aa64isar0,
            ac_aa64isar1: ffi.aa64isar1,
            ac_aa64mmfr0: ffi.aa64mmfr0,
            ac_aa64mmfr1: ffi.aa64mmfr1,
            ac_aa64mmfr2: ffi.aa64mmfr2,
            ac_aa64pfr0: ffi.aa64pfr0,
            ac_aa64pfr1: ffi.aa64pfr1,
            ac_aa64zfr0: ffi.aa64zfr0,
            ac_mvfr0: ffi.mvfr0,
            ac_mvfr1: ffi.mvfr1,
            ac_mvfr2: ffi.mvfr2,
            ac_pad: ffi.pad,
            ac_clidr: ffi.clidr,
            ac_ctr: ffi.ctr,
        };
    };
    #[cfg(target_os = "openbsd")]
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::no_effect_underscore_binding
    )]
    const _: fn() = || {
        use imp::ffi;
        use test_helper::{libc, sys};
        let mut _sysctl: unsafe extern "C" fn(
            *const ffi::c_int,
            ffi::c_uint,
            *mut ffi::c_void,
            *mut ffi::c_size_t,
            *mut ffi::c_void,
            ffi::c_size_t,
        ) -> ffi::c_int = ffi::sysctl;
        _sysctl = libc::sysctl;
        _sysctl = sys::sysctl;
        static_assert!(ffi::CTL_MACHDEP == libc::CTL_MACHDEP);
        static_assert!(ffi::CTL_MACHDEP == sys::CTL_MACHDEP as ffi::c_int);
        // static_assert!(ffi::CPU_ID_AA64ISAR0 == libc::CPU_ID_AA64ISAR0); // libc doesn't have this
        static_assert!(ffi::CPU_ID_AA64ISAR0 == sys::CPU_ID_AA64ISAR0 as ffi::c_int);
        // static_assert!(ffi::CPU_ID_AA64ISAR1 == libc::CPU_ID_AA64ISAR1); // libc doesn't have this
        static_assert!(ffi::CPU_ID_AA64ISAR1 == sys::CPU_ID_AA64ISAR1 as ffi::c_int);
        // static_assert!(ffi::CPU_ID_AA64MMFR2 == libc::CPU_ID_AA64MMFR2); // libc doesn't have this
        static_assert!(ffi::CPU_ID_AA64MMFR2 == sys::CPU_ID_AA64MMFR2 as ffi::c_int);
    };
}
