// SPDX-License-Identifier: Apache-2.0 OR MIT

// The rustc-cfg emitted by the build script are *not* public API.

#![allow(clippy::match_same_arms)] // https://github.com/rust-lang/rust-clippy/issues/12044

#[path = "version.rs"]
mod version;
use version::{rustc_version, Version};

use std::{env, str};

include!("no_atomic.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=no_atomic.rs");
    println!("cargo:rerun-if-changed=version.rs");

    #[cfg(feature = "unsafe-assume-single-core")]
    println!("cargo:rustc-cfg=portable_atomic_unsafe_assume_single_core");
    #[cfg(feature = "s-mode")]
    println!("cargo:rustc-cfg=portable_atomic_s_mode");
    #[cfg(feature = "force-amo")]
    println!("cargo:rustc-cfg=portable_atomic_force_amo");
    #[cfg(feature = "disable-fiq")]
    println!("cargo:rustc-cfg=portable_atomic_disable_fiq");

    let target = &*env::var("TARGET").expect("TARGET not set");
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let target_os = &*env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    let version = match rustc_version() {
        Some(version) => version,
        None => {
            if env::var_os("PORTABLE_ATOMIC_DENY_WARNINGS").unwrap_or_default() == "1" {
                panic!("unable to determine rustc version")
            }
            println!(
                "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc (1.{})",
                env!("CARGO_PKG_NAME"),
                Version::LATEST.minor
            );
            Version::LATEST
        }
    };

    if version.minor >= 80 {
        println!(
            r#"cargo:rustc-check-cfg=cfg(target_feature,values("lse2","lse128","rcpc3","quadword-atomics","fast-serialization","load-store-on-cond","distinct-ops","miscellaneous-extensions-3"))"#
        );

        // Custom cfgs set by build script. Not public API.
        // grep -E 'cargo:rustc-cfg=' build.rs | grep -v '=//' | sed -E 's/^.*cargo:rustc-cfg=//; s/(=\\)?".*$//' | LC_ALL=C sort -u | tr '\n' ','
        println!(
            "cargo:rustc-check-cfg=cfg(portable_atomic_disable_fiq,portable_atomic_force_amo,portable_atomic_ll_sc_rmw,portable_atomic_llvm_15,portable_atomic_llvm_16,portable_atomic_llvm_18,portable_atomic_new_atomic_intrinsics,portable_atomic_no_asm,portable_atomic_no_asm_maybe_uninit,portable_atomic_no_atomic_64,portable_atomic_no_atomic_cas,portable_atomic_no_atomic_load_store,portable_atomic_no_atomic_min_max,portable_atomic_no_cfg_target_has_atomic,portable_atomic_no_cmpxchg16b_intrinsic,portable_atomic_no_cmpxchg16b_target_feature,portable_atomic_no_const_raw_ptr_deref,portable_atomic_no_const_transmute,portable_atomic_no_core_unwind_safe,portable_atomic_no_stronger_failure_ordering,portable_atomic_no_track_caller,portable_atomic_no_unsafe_op_in_unsafe_fn,portable_atomic_s_mode,portable_atomic_sanitize_thread,portable_atomic_target_feature,portable_atomic_unsafe_assume_single_core,portable_atomic_unstable_asm,portable_atomic_unstable_asm_experimental_arch,portable_atomic_unstable_cfg_target_has_atomic,portable_atomic_unstable_isa_attribute)"
        );
        // TODO: handle multi-line target_feature_fallback
        // grep -E 'target_feature_fallback\("' build.rs | sed -E 's/^.*target_feature_fallback\(//; s/",.*$/"/' | LC_ALL=C sort -u | tr '\n' ','
        println!(
            r#"cargo:rustc-check-cfg=cfg(portable_atomic_target_feature,values("cmpxchg16b","distinct-ops","fast-serialization","load-store-on-cond","lse","lse128","lse2","mclass","miscellaneous-extensions-3","quadword-atomics","rcpc3","v6"))"#
        );
    }

    // https://github.com/rust-lang/rust/pull/123745 (includes https://github.com/rust-lang/cargo/pull/13560) merged in Rust 1.79 (nightly-2024-04-11).
    if !version.probe(79, 2024, 4, 10) {
        // HACK: If --target is specified, rustflags is not applied to the build
        // script itself, so the build script will not be recompiled when rustflags
        // is changed. That in itself is not a problem, but the old Cargo does
        // not rerun the build script as well, which can be problematic.
        // https://github.com/rust-lang/cargo/issues/13003
        // This problem has been fixed in 1.79 so only older versions need a workaround.
        println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
        println!("cargo:rerun-if-env-changed=RUSTFLAGS");
        println!("cargo:rerun-if-env-changed=CARGO_BUILD_RUSTFLAGS");
        let mut target_upper = target.replace(|c: char| c == '-' || c == '.', "_");
        target_upper.make_ascii_uppercase();
        println!("cargo:rerun-if-env-changed=CARGO_TARGET_{}_RUSTFLAGS", target_upper);
    }

    // Note that this is `no_`*, not `has_*`. This allows treating as the latest
    // stable rustc is used when the build script doesn't run. This is useful
    // for non-cargo build systems that don't run the build script.
    // atomic_min_max stabilized in Rust 1.45 (nightly-2020-05-30): https://github.com/rust-lang/rust/pull/72324
    if !version.probe(45, 2020, 5, 29) {
        println!("cargo:rustc-cfg=portable_atomic_no_atomic_min_max");
    }
    // track_caller stabilized in Rust 1.46 (nightly-2020-07-02): https://github.com/rust-lang/rust/pull/72445
    if !version.probe(46, 2020, 7, 1) {
        println!("cargo:rustc-cfg=portable_atomic_no_track_caller");
    }
    // unsafe_op_in_unsafe_fn stabilized in Rust 1.52 (nightly-2021-03-11): https://github.com/rust-lang/rust/pull/79208
    if !version.probe(52, 2021, 3, 10) {
        println!("cargo:rustc-cfg=portable_atomic_no_unsafe_op_in_unsafe_fn");
    }
    // const_transmute stabilized in Rust 1.56 (nightly-2021-07-29): https://github.com/rust-lang/rust/pull/85769
    if !version.probe(56, 2021, 7, 28) {
        println!("cargo:rustc-cfg=portable_atomic_no_const_transmute");
    }
    // https://github.com/rust-lang/rust/pull/84662 merged in Rust 1.56 (nightly-2021-08-02).
    if !version.probe(56, 2021, 8, 1) {
        println!("cargo:rustc-cfg=portable_atomic_no_core_unwind_safe");
    }
    // const_raw_ptr_deref stabilized in Rust 1.58 (nightly-2021-11-15): https://github.com/rust-lang/rust/pull/89551
    if !version.probe(58, 2021, 11, 14) {
        println!("cargo:rustc-cfg=portable_atomic_no_const_raw_ptr_deref");
    }
    // https://github.com/rust-lang/rust/pull/98383 merged in Rust 1.64 (nightly-2022-07-19).
    if !version.probe(64, 2022, 7, 18) {
        println!("cargo:rustc-cfg=portable_atomic_no_stronger_failure_ordering");
    }
    // https://github.com/rust-lang/rust/pull/114790 merged in nightly-2023-08-24
    if !version.probe(74, 2023, 8, 23) {
        println!("cargo:rustc-cfg=portable_atomic_no_asm_maybe_uninit");
    }

    // asm stabilized in Rust 1.59 (nightly-2021-12-16): https://github.com/rust-lang/rust/pull/91728
    let no_asm = !version.probe(59, 2021, 12, 15);
    if no_asm {
        if version.nightly
            && version.probe(46, 2020, 6, 20)
            && ((target_arch != "x86" && target_arch != "x86_64") || version.llvm >= 10)
            && is_allowed_feature("asm")
        {
            // This feature was added in Rust 1.45 (nightly-2020-05-20), but
            // concat! in asm! requires Rust 1.46 (nightly-2020-06-21).
            // x86 intel syntax requires LLVM 10 (since Rust 1.53, the minimum
            // external LLVM version is 10+: https://github.com/rust-lang/rust/pull/83387).
            // The part of this feature we use has not been changed since nightly-2020-06-21
            // until it was stabilized in nightly-2021-12-16, so it can be safely enabled in
            // nightly, which is older than nightly-2021-12-16.
            println!("cargo:rustc-cfg=portable_atomic_unstable_asm");
        }
        println!("cargo:rustc-cfg=portable_atomic_no_asm");
    }

    // feature(cfg_target_has_atomic) stabilized in Rust 1.60 (nightly-2022-02-11): https://github.com/rust-lang/rust/pull/93824
    if !version.probe(60, 2022, 2, 10) {
        if version.nightly
            && version.probe(40, 2019, 10, 13)
            && is_allowed_feature("cfg_target_has_atomic")
        {
            // This feature has not been changed since the change in Rust 1.40 (nightly-2019-10-14)
            // until it was stabilized in nightly-2022-02-11, so it can be safely enabled in
            // nightly, which is older than nightly-2022-02-11.
            println!("cargo:rustc-cfg=portable_atomic_unstable_cfg_target_has_atomic");
        } else {
            println!("cargo:rustc-cfg=portable_atomic_no_cfg_target_has_atomic");
            let target = &*convert_custom_linux_target(target);
            if NO_ATOMIC_CAS.contains(&target) {
                println!("cargo:rustc-cfg=portable_atomic_no_atomic_cas");
            }
            if NO_ATOMIC_64.contains(&target) {
                println!("cargo:rustc-cfg=portable_atomic_no_atomic_64");
            } else {
                // Otherwise, assuming `"max-atomic-width" == 64` or `"max-atomic-width" == 128`.
            }
        }
    }
    // We don't need to use convert_custom_linux_target here because all linux targets have atomics.
    if NO_ATOMIC.contains(&target) {
        println!("cargo:rustc-cfg=portable_atomic_no_atomic_load_store");
    }

    if version.llvm >= 16 {
        println!("cargo:rustc-cfg=portable_atomic_llvm_16");
    }
    if version.nightly {
        // `cfg(sanitize = "..")` is not stabilized.
        let sanitize = env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
        if sanitize.contains("thread") {
            // Most kinds of sanitizers are not compatible with asm
            // (https://github.com/google/sanitizers/issues/192),
            // but it seems that ThreadSanitizer is the only one that can cause
            // false positives in our code.
            println!("cargo:rustc-cfg=portable_atomic_sanitize_thread");
        }

        // https://github.com/rust-lang/rust/pull/93868 merged in Rust 1.60 (nightly-2022-02-13).
        // https://github.com/rust-lang/rust/pull/111331 merged in Rust 1.71 (nightly-2023-05-09).
        if !no_asm
            && (target_arch == "powerpc64" && version.probe(60, 2022, 2, 12)
                || target_arch == "s390x" && version.probe(71, 2023, 5, 8))
            && is_allowed_feature("asm_experimental_arch")
        {
            println!("cargo:rustc-cfg=portable_atomic_unstable_asm_experimental_arch");
        }
    }

    let is_macos = target_os == "macos";
    let is_apple = is_macos
        || target_os == "ios"
        || target_os == "tvos"
        || target_os == "watchos"
        || target_os == "visionos";
    match target_arch {
        "x86_64" => {
            // cmpxchg16b_target_feature stabilized in Rust 1.69 (nightly-2023-03-01): https://github.com/rust-lang/rust/pull/106774
            if !version.probe(69, 2023, 2, 28) {
                println!("cargo:rustc-cfg=portable_atomic_no_cmpxchg16b_target_feature");
            }
            // For Miri and ThreadSanitizer.
            // https://github.com/rust-lang/rust/pull/109359 (includes https://github.com/rust-lang/stdarch/pull/1358) merged in Rust 1.70 (nightly-2023-03-24).
            if version.nightly && !version.probe(70, 2023, 3, 23) {
                println!("cargo:rustc-cfg=portable_atomic_no_cmpxchg16b_intrinsic");
            }

            // cmpxchg16b_target_feature stabilized in Rust 1.69.
            if needs_target_feature_fallback(&version, Some(69)) {
                // x86_64 Apple targets always support CMPXCHG16B:
                // https://github.com/rust-lang/rust/blob/1.68.0/compiler/rustc_target/src/spec/x86_64_apple_darwin.rs#L8
                // https://github.com/rust-lang/rust/blob/1.68.0/compiler/rustc_target/src/spec/apple_base.rs#L69-L70
                // (Since Rust 1.78, Windows (except Windows 7) targets also enable CMPXCHG16B, but
                // this branch is only used on pre-1.69 that cmpxchg16b_target_feature is unstable.)
                // Script to get targets that support cmpxchg16b by default:
                // $ (for target in $(rustc --print target-list); do [[ "${target}" == "x86_64"* ]] && rustc --print cfg --target "${target}" | grep -q cmpxchg16b && echo "${target}"; done)
                let has_cmpxchg16b = is_apple;
                // LLVM recognizes this also as cx16 target feature: https://godbolt.org/z/KM3jz616j
                // However, it is unlikely that rustc will support that name, so we ignore it.
                target_feature_fallback("cmpxchg16b", has_cmpxchg16b);
            }
        }
        "aarch64" => {
            // For Miri and ThreadSanitizer.
            // https://github.com/rust-lang/rust/pull/97423 merged in Rust 1.64 (nightly-2022-06-30).
            if version.nightly && version.probe(64, 2022, 6, 29) {
                println!("cargo:rustc-cfg=portable_atomic_new_atomic_intrinsics");
            }

            // aarch64 macOS always supports FEAT_LSE and FEAT_LSE2 because it is armv8.5-a:
            // https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/include/llvm/TargetParser/AArch64TargetParser.h#L728
            let mut has_lse = is_macos;
            // FEAT_LSE2 doesn't imply FEAT_LSE. FEAT_LSE128 implies FEAT_LSE but not FEAT_LSE2.
            // As of rustc 1.78, target_feature "lse2"/"lse128"/"rcpc3" is not available on rustc side:
            // https://github.com/rust-lang/rust/blob/1.78.0/compiler/rustc_target/src/target_features.rs#L87
            target_feature_fallback("lse2", is_macos);
            // LLVM supports FEAT_LRCPC3 and FEAT_LSE128 on LLVM 16+:
            // https://github.com/llvm/llvm-project/commit/a6aaa969f7caec58a994142f8d855861cf3a1463
            // https://github.com/llvm/llvm-project/commit/7fea6f2e0e606e5339c3359568f680eaf64aa306
            if version.llvm >= 16 {
                has_lse |= target_feature_fallback("lse128", false);
                target_feature_fallback("rcpc3", false);
            }
            // aarch64_target_feature stabilized in Rust 1.61.
            if needs_target_feature_fallback(&version, Some(61)) {
                target_feature_fallback("lse", has_lse);
            }

            // As of Apple M1/M1 Pro, on Apple hardware, CAS-loop-based RMW is much slower than
            // LL/SC-loop-based RMW: https://github.com/taiki-e/portable-atomic/pull/89
            if is_apple || target_cpu().map_or(false, |cpu| cpu.starts_with("apple-")) {
                println!("cargo:rustc-cfg=portable_atomic_ll_sc_rmw");
            }
        }
        "arm" => {
            // For non-Linux/Android pre-v6 ARM (tier 3) with unsafe_assume_single_core enabled.
            // feature(isa_attribute) stabilized in Rust 1.67 (nightly-2022-11-06): https://github.com/rust-lang/rust/pull/102458
            if version.nightly && !version.probe(67, 2022, 11, 5) {
                println!("cargo:rustc-cfg=portable_atomic_unstable_isa_attribute");
            }

            if needs_target_feature_fallback(&version, None) {
                // #[cfg(target_feature = "v7")] and others don't work on stable.
                // armv7-unknown-linux-gnueabihf
                //    ^^
                let mut subarch =
                    strip_prefix(target, "arm").or_else(|| strip_prefix(target, "thumb")).unwrap();
                subarch = strip_prefix(subarch, "eb").unwrap_or(subarch); // ignore endianness
                subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
                subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
                let mut known = true;
                // See https://github.com/taiki-e/atomic-maybe-uninit/blob/HEAD/build.rs for details
                let mut mclass = false;
                match subarch {
                    "v7" | "v7a" | "v7neon" | "v7s" | "v7k" | "v8" | "v8a" | "v9" | "v9a" => {} // aclass
                    "v7r" | "v8r" | "v9r" => {} // rclass
                    "v6m" | "v7em" | "v7m" | "v8m" => mclass = true,
                    // arm-linux-androideabi is v5te
                    // https://github.com/rust-lang/rust/blob/1.78.0/compiler/rustc_target/src/spec/targets/arm_linux_androideabi.rs#L18
                    _ if target == "arm-linux-androideabi" => subarch = "v5te",
                    // armeb-unknown-linux-gnueabi is v8 & aclass
                    // https://github.com/rust-lang/rust/blob/1.78.0/compiler/rustc_target/src/spec/targets/armeb_unknown_linux_gnueabi.rs#L18
                    _ if target == "armeb-unknown-linux-gnueabi" => subarch = "v8",
                    // Legacy arm architectures (pre-v7 except v6m) don't have *class target feature.
                    "" => subarch = "v6",
                    "v4t" | "v5te" | "v6" | "v6k" => {}
                    _ => {
                        known = false;
                        if env::var_os("PORTABLE_ATOMIC_DENY_WARNINGS").unwrap_or_default() == "1" {
                            panic!("unrecognized arm subarch: {}", target)
                        }
                        println!(
                            "cargo:warning={}: unrecognized arm subarch: {}",
                            env!("CARGO_PKG_NAME"),
                            target
                        );
                    }
                }
                let v6 = known
                    && (subarch.starts_with("v6")
                        || subarch.starts_with("v7")
                        || subarch.starts_with("v8")
                        || subarch.starts_with("v9"));
                target_feature_fallback("v6", v6);
                target_feature_fallback("mclass", mclass);
            }
        }
        "powerpc64" => {
            // For Miri and ThreadSanitizer.
            if version.nightly && version.llvm >= 15 {
                println!("cargo:rustc-cfg=portable_atomic_llvm_15");
            }

            let target_endian =
                env::var("CARGO_CFG_TARGET_ENDIAN").expect("CARGO_CFG_TARGET_ENDIAN not set");
            // powerpc64le is pwr8+ by default https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/lib/Target/PowerPC/PPC.td#L674
            // See also https://github.com/rust-lang/rust/issues/59932
            let mut has_pwr8_features = target_endian == "little";
            // https://github.com/llvm/llvm-project/commit/549e118e93c666914a1045fde38a2cac33e1e445
            if let Some(cpu) = &target_cpu() {
                if let Some(mut cpu_version) = strip_prefix(cpu, "pwr") {
                    cpu_version = strip_suffix(cpu_version, "x").unwrap_or(cpu_version); // for pwr5x and pwr6x
                    if let Ok(cpu_version) = cpu_version.parse::<u32>() {
                        has_pwr8_features = cpu_version >= 8;
                    }
                } else {
                    // https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/lib/Target/PowerPC/PPC.td#L674
                    // https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/lib/Target/PowerPC/PPC.td#L456
                    has_pwr8_features = cpu == "ppc64le" || cpu == "future";
                }
            }
            // As of rustc 1.78, target_feature "quadword-atomics" is not available on rustc side:
            // https://github.com/rust-lang/rust/blob/1.78.0/compiler/rustc_target/src/target_features.rs#L255
            // lqarx and stqcx.
            target_feature_fallback("quadword-atomics", has_pwr8_features);
        }
        "s390x" => {
            // For Miri and ThreadSanitizer.
            if version.nightly && version.llvm >= 18 {
                println!("cargo:rustc-cfg=portable_atomic_llvm_18");
            }

            // https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/lib/Target/SystemZ/SystemZFeatures.td
            let mut arch9_features = false; // z196+
            let mut arch13_features = false; // z15+
            if let Some(cpu) = target_cpu() {
                // https://github.com/llvm/llvm-project/blob/llvmorg-18.1.2/llvm/lib/Target/SystemZ/SystemZProcessors.td
                match &*cpu {
                    "arch9" | "z196" | "arch10" | "zEC12" | "arch11" | "z13" | "arch12" | "z14" => {
                        arch9_features = true;
                    }
                    "arch13" | "z15" | "arch14" | "z16" => {
                        arch9_features = true;
                        arch13_features = true;
                    }
                    _ => {}
                }
            }
            // As of rustc 1.78, target_feature "fast-serialization"/"load-store-on-cond"/"distinct-ops"/"miscellaneous-extensions-3" is not available on rustc side:
            // https://github.com/rust-lang/rust/blob/1.78.0/compiler/rustc_target/src/target_features.rs
            // bcr 14,0
            target_feature_fallback("fast-serialization", arch9_features);
            // {l,st}oc{,g}{,r}
            target_feature_fallback("load-store-on-cond", arch9_features);
            // {al,sl,n,o,x}{,g}rk
            target_feature_fallback("distinct-ops", arch9_features);
            // nand (nnr{,g}k), select (sel{,g}r), etc.
            target_feature_fallback("miscellaneous-extensions-3", arch13_features);
        }
        _ => {}
    }
}

// HACK: Currently, it seems that the only way to handle unstable target
// features on the stable is to parse the `-C target-feature` in RUSTFLAGS.
//
// - #[cfg(target_feature = "unstable_target_feature")] doesn't work on stable.
// - CARGO_CFG_TARGET_FEATURE excludes unstable target features on stable.
//
// As mentioned in the [RFC2045], unstable target features are also passed to LLVM
// (e.g., https://godbolt.org/z/4rr7rMcfG), so this hack works properly on stable.
//
// [RFC2045]: https://rust-lang.github.io/rfcs/2045-target-feature.html#backend-compilation-options
fn needs_target_feature_fallback(version: &Version, stable: Option<u32>) -> bool {
    match stable {
        // In these cases, cfg(target_feature = "...") would work, so skip emitting our own fallback target_feature cfg.
        _ if version.nightly => false,
        Some(stabilized) if version.minor >= stabilized => false,
        _ => true,
    }
}
fn target_feature_fallback(name: &str, mut has_target_feature: bool) -> bool {
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = strip_prefix(flag, "-C").unwrap_or(flag);
            if let Some(flag) = strip_prefix(flag, "target-feature=") {
                for s in flag.split(',') {
                    // TODO: Handles cases where a specific target feature
                    // implicitly enables another target feature.
                    match (s.as_bytes().first(), s.as_bytes().get(1..)) {
                        (Some(b'+'), Some(f)) if f == name.as_bytes() => has_target_feature = true,
                        (Some(b'-'), Some(f)) if f == name.as_bytes() => has_target_feature = false,
                        _ => {}
                    }
                }
            }
        }
    }
    if has_target_feature {
        println!("cargo:rustc-cfg=portable_atomic_target_feature=\"{}\"", name);
    }
    has_target_feature
}

fn target_cpu() -> Option<String> {
    let rustflags = env::var_os("CARGO_ENCODED_RUSTFLAGS")?;
    let rustflags = rustflags.to_string_lossy();
    let mut cpu = None;
    for mut flag in rustflags.split('\x1f') {
        flag = strip_prefix(flag, "-C").unwrap_or(flag);
        if let Some(flag) = strip_prefix(flag, "target-cpu=") {
            cpu = Some(flag);
        }
    }
    cpu.map(str::to_owned)
}

fn is_allowed_feature(name: &str) -> bool {
    // https://github.com/dtolnay/thiserror/pull/248
    if env::var_os("RUSTC_STAGE").is_some() {
        return false;
    }

    // allowed by default
    let mut allowed = true;
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = strip_prefix(flag, "-Z").unwrap_or(flag);
            if let Some(flag) = strip_prefix(flag, "allow-features=") {
                // If it is specified multiple times, the last value will be preferred.
                allowed = flag.split(',').any(|allowed| allowed == name);
            }
        }
    }
    allowed
}

// Adapted from https://github.com/crossbeam-rs/crossbeam/blob/crossbeam-utils-0.8.14/build-common.rs.
//
// The target triplets have the form of 'arch-vendor-system'.
//
// When building for Linux (e.g. the 'system' part is
// 'linux-something'), replace the vendor with 'unknown'
// so that mapping to rust standard targets happens correctly.
fn convert_custom_linux_target(target: &str) -> String {
    let mut parts: Vec<&str> = target.split('-').collect();
    let system = parts.get(2);
    if system == Some(&"linux") {
        parts[1] = "unknown";
    }
    parts.join("-")
}

// str::strip_prefix requires Rust 1.45
#[must_use]
fn strip_prefix<'a>(s: &'a str, pat: &str) -> Option<&'a str> {
    if s.starts_with(pat) {
        Some(&s[pat.len()..])
    } else {
        None
    }
}
// str::strip_suffix requires Rust 1.45
#[must_use]
fn strip_suffix<'a>(s: &'a str, pat: &str) -> Option<&'a str> {
    if s.ends_with(pat) {
        Some(&s[..s.len() - pat.len()])
    } else {
        None
    }
}
