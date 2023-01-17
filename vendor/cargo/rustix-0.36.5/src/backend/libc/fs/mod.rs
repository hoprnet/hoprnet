#[cfg(not(target_os = "redox"))]
pub(crate) mod dir;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "solaris",
    target_os = "wasi",
)))]
pub(crate) mod makedev;
#[cfg(not(windows))]
pub(crate) mod syscalls;
pub(crate) mod types;
