//! Control console coloring across all dependencies
//!
//! # Motivation
//!
//! Detecting a terminal's color capabilities and passing it down to each writer
//! can be obnoxious.  Some crates try to make this easier by detecting the environment for you and
//! making their own choice to print colors.  As an application author, you own the experience for
//! your application and want the behavior to be consistent.  To get this, you have to dig into
//! each crate's implementation to see how they auto-detect color capabilities and, if they don't
//! do it how you want, hope they provide a way to override it so you can implement it yourself.
//!
//! Like with logging, your terminal's capabilities and how to treat it is a behavior that cuts
//! across your application.  So to make things more consistent and easier to control,
//! `concolor` introduces shared detection logic that all crates can call into to get
//! consistent behavior.  The application author can then choose what feature flags are enabled to
//! decide on what the end-user experience should be.
//!
//! # `[[bin]]`s
//!
//! ```toml
//! [dependencies]
//! concolor = { version = "0.0.11", features = "color" }
//! ```
//! Notes:
//! - With the
//!   [2021 edition / `resolver = "2"`](https://doc.rust-lang.org/nightly/edition-guide/rust-2021/default-cargo-resolver.html),
//!   you will also need to specify this in your `build-dependencies` if you want `build.rs` to have color
//!   as well.
//!
//! If you are providing a command line option for controlling color, just call
//! ```rust
//! let when = concolor::ColorChoice::Always;
//! concolor::set(when);
//! ```
//!
//! See also [`concolor-clap`](https://docs.rs/concolor-clap)
//!
//! # `[lib]`s
//!
//! The `[[bin]]` is responsible for defining the policy of how colors are determined, so to depend
//! on `concolor`:
//! ```toml
//! [dependencies]
//! concolor = { version = "0.0.11", default-features = false }
//! ```
//!
//! At times, you might want to provide a convenience feature for color support, so you could also:
//! ```toml
//! [features]
//! default = ["color"]
//! color = "concolor/auto"
//!
//! [dependencies]
//! concolor = { version = "0.0.11", optional = True}
//! ```
//! Notes:
//! - Your choice on whether to make this default or not
//! - Depending on your context, name it either `color` (for a crate like `clap`) or `auto` (for a
//!   crate like `termcolor`)
//!
//! Then just ask as needed:
//! ```rust
//! let stdout_support = concolor::get(concolor::Stream::Stdout);
//! if stdout_support.ansi_color() {
//!     // Output ANSI escape sequences
//!     if stdout_support.truecolor() {
//!         // Get even fancier with the colors
//!     }
//! } else if stdout_support.color() {
//!     // Legacy Windows version, control the console as needed
//! } else {
//!     // No coloring
//! }
//! ```
//!
//! # Features
//!
//! - `auto`: Guess color status based on all possible sources, including:
//!   - `api_unstable`: Allow controlling color via the API (until 1.0, this is not guaranteed to
//!      work across crates which is why this is `_unstable`)
//!   - `interactive`: Check if stdout/stderr is a TTY
//!   - `clicolor`: Respect [CLICOLOR] spec
//!   - `no_color`: Respect [NO_COLOR] spec
//!   - `term`: Check `TERM`
//!   - `windows`: Check if we can enable ANSI support
//!
//! [CLICOLOR]: https://bixense.com/clicolors/
//! [NO_COLOR]: https://no-color.org/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "core")]
mod color;
#[cfg(feature = "core")]
pub use color::*;

#[cfg(not(feature = "core"))]
mod no_color;
#[cfg(not(feature = "core"))]
pub use no_color::*;

mod choice;
pub use choice::*;
mod stream;
pub use stream::*;
