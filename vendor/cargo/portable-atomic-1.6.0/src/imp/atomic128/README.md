# Implementation of 128-bit atomics

## 128-bit atomics instructions

Here is the table of targets that support 128-bit atomics and the instructions used:

| target_arch | load | store | CAS | RMW | note |
| ----------- | ---- | ----- | --- | --- | ---- |
| x86_64 | cmpxchg16b or vmovdqa | cmpxchg16b or vmovdqa | cmpxchg16b | cmpxchg16b | cmpxchg16b target feature required. vmovdqa requires Intel or AMD CPU with AVX. <br> Both compile-time and run-time detection are supported for cmpxchg16b. vmovdqa is currently run-time detection only. <br> Requires rustc 1.59+ when cmpxchg16b target feature is enabled at compile-time, otherwise requires rustc 1.69+ |
| aarch64 | ldxp/stxp or casp or ldp/ldiapp | ldxp/stxp or casp or stp/stilp/swpp | ldxp/stxp or casp | ldxp/stxp or casp/swpp/ldclrp/ldsetp | casp requires lse target feature, ldp/stp requires lse2 target feature, ldiapp/stilp requires lse2 and rcpc3 target features, swpp/ldclrp/ldsetp requires lse128 target feature. <br> Both compile-time and run-time detection are supported for lse and lse2. Others are currently compile-time detection only. <br> Requires rustc 1.59+ |
| powerpc64 | lq | stq | lqarx/stqcx. | lqarx/stqcx. | Requires target-cpu pwr8+ (powerpc64le is pwr8 by default). Both compile-time and run-time detection are supported (run-time detection is currently disabled by default). <br> Requires nightly |
| s390x | lpq | stpq | cdsg | cdsg | Requires nightly |

On compiler versions or platforms where these are not supported, the fallback implementation is used.

See [aarch64.rs](aarch64.rs) module-level comments for more details on the instructions used on aarch64.

## Comparison with core::intrinsics::atomic_\* (core::sync::atomic::Atomic{I,U}128)

This directory has target-specific implementations with inline assembly ([aarch64.rs](aarch64.rs), [x86_64.rs](x86_64.rs), [powerpc64.rs](powerpc64.rs), [s390x.rs](s390x.rs)) and an implementation without inline assembly ([intrinsics.rs](intrinsics.rs)). The latter currently always needs nightly compilers and is only used for Miri and ThreadSanitizer, which do not support inline assembly.

Implementations with inline assembly generate assemblies almost equivalent to the `core::intrinsics::atomic_*` (used in `core::sync::atomic::Atomic{I,U}128`) for many operations, but some operations may or may not generate more efficient code. For example:

- On x86_64, implementation with inline assembly contains additional optimizations (e.g., [#16](https://github.com/taiki-e/portable-atomic/pull/16)) and is much faster for some operations.
- On aarch64, implementation with inline assembly supports outline-atomics on more operating systems, and may be faster in environments where outline-atomics can improve performance.
- On powerpc64 and s390x, LLVM does not support generating some 128-bit atomic operations (see [intrinsics.rs](intrinsics.rs) module-level comments), and we use CAS loop to implement them, so implementation with inline assembly may be faster for those operations.
- In implementations without inline assembly, the compiler may reuse condition flags that have changed as a result of the operation, or use immediate values instead of registers, depending on the situation.

As 128-bit atomics-related APIs stabilize in the standard library, implementations with inline assembly are planned to be updated to get the benefits of both.

## Run-time feature detection

[detect](detect) module has run-time feature detection implementations.

Here is the table of targets that support run-time feature detection and the instruction or API used:

| target_arch | target_os/target_env | instruction/API | features | note |
| ----------- | -------------------- | --------------- | -------- | ---- |
| x86_64      | all (except for sgx) | cpuid           | all      | Enabled by default |
| aarch64     | linux                | getauxval       | all      | Only enabled by default on `*-linux-gnu*`, and `*-linux-musl*"` (default is static linking)/`*-linux-ohos*` (default is dynamic linking) with dynamic linking enabled. |
| aarch64     | android              | getauxval       | all      | Enabled by default |
| aarch64     | freebsd              | elf_aux_info    | lse, lse2 | Enabled by default |
| aarch64     | netbsd               | sysctl          | all      | Enabled by default |
| aarch64     | openbsd              | sysctl          | lse      | Enabled by default |
| aarch64     | macos                | sysctl          | all      | Currently only used in tests because FEAT_LSE and FEAT_LSE2 are always available at compile-time. |
| aarch64     | windows              | IsProcessorFeaturePresent | lse | Enabled by default |
| aarch64     | fuchsia              | zx_system_get_features | lse | Enabled by default |
| powerpc64   | linux                | getauxval       | all      | Disabled by default |
| powerpc64   | freebsd              | elf_aux_info    | all      | Disabled by default |

Run-time detection is enabled by default on most targets and can be disabled with `--cfg portable_atomic_no_outline_atomics`.

On some targets, run-time detection is disabled by default mainly for compatibility with older versions of operating systems or incomplete build environments, and can be enabled by `--cfg portable_atomic_outline_atomics`. (When both cfg are enabled, `*_no_*` cfg is preferred.)

For targets not included in the above table, run-time detection is always disabled and works the same as when `--cfg portable_atomic_no_outline_atomics` is set.

See [detect/auxv.rs](detect/auxv.rs) module-level comments for more details on Linux/Android/FreeBSD.

See also [docs on `portable_atomic_no_outline_atomics`](https://github.com/taiki-e/portable-atomic/blob/HEAD/README.md#optional-cfg-no-outline-atomics) in the top-level readme.
