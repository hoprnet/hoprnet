//! `os_info`
//!
//! Provides interfaces for getting information about the current operating system, such as type,
//! version, edition and bitness.

#![deny(
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    missing_doc_code_examples
)]

#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
mod imp;

#[cfg(target_os = "emscripten")]
#[path = "emscripten/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(target_os = "redox")]
#[path = "redox/mod.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(not(any(
    target_os = "android",
    target_os = "emscripten",
    target_os = "linux",
    target_os = "macos",
    target_os = "redox",
    target_os = "windows"
)))]
#[path = "unknown/mod.rs"]
mod imp;

mod bitness;
mod info;
#[cfg(not(windows))]
mod matcher;
mod os_type;
mod version;

pub use bitness::Bitness;
pub use info::Info;
pub use os_type::Type;
pub use version::{Version, VersionType};

/// Returns information about the current operating system (type, version, edition, etc.).
///
/// # Examples
///
/// ```
/// use os_info;
///
/// let info = os_info::get();
///
/// // Print full information:
/// println!("OS information: {}", info);
///
/// // Print information separately:
/// println!("Type: {}", info.os_type());
/// println!("Version: {}", info.version());
/// println!("Bitness: {}", info.bitness());
/// ```
pub fn get() -> Info {
    imp::current_platform()
}
