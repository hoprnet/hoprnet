//! This library implements basic [clicolor](http://bixense.com/clicolors/) control for
//! other rust libraries.  The idea is that other crates can depend on this to have a
//! central source of truth for the colorization of command line applications.
//!
//! it follows the cli color specification:
//!
//! * `CLICOLOR != 0`: ANSI colors are supported and should be used when the program isn't piped.
//! * `CLICOLOR == 0`: Don't output ANSI color escape codes.
//! * `CLICOLOR_FORCE != 0`: ANSI colors should be enabled no matter what.
//!
//! ## Example Usage
//!
//! ```rust
//! extern crate clicolors_control;
//!
//! pub fn main() {
//!     if clicolors_control::colors_enabled() {
//!         println!("\x1b[36mThis is colored text.\x1b[0m");
//!     } else {
//!         println!("Someone turned off the colors :()")
//!     }
//! }
//! ```
//!
//! ## Controlling Colors
//!
//! Colors can be turned on and off for the current process with `set_colors_enabled`.
//!
//! ## Windows 10 Console
//!
//! The default behavior of this crate is to reconfigure the windows console to enable the
//! VT100 emulation when available the first time colors are requested.  This will only work
//! on recent Windows 10 versions.  This feature can be disabled by removing the default
//! `terminal_autoconfig` feature.
//!
//! The terminal can be manually configured for colors by calling `configure_terminal()`

#[cfg(unix)] extern crate libc;
#[cfg(windows)] extern crate winapi;
#[cfg(windows)] extern crate kernel32;
#[macro_use] extern crate lazy_static;

mod common;
#[cfg(unix)] mod unix;
#[cfg(windows)] mod windows;

pub use common::{colors_enabled, set_colors_enabled, configure_terminal};
