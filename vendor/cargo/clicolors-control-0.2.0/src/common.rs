use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

/// Returns the default value for `colors_enabled`.
pub fn enable_colors_by_default() -> bool {
    (is_a_color_terminal() && &env::var("CLICOLOR").unwrap_or("1".into()) != "0") ||
    &env::var("CLICOLOR_FORCE").unwrap_or("0".into()) != "0"
}

lazy_static! {
    static ref ENABLE_COLORS: AtomicBool =
        AtomicBool::new(enable_colors_by_default());
}

/// Returns `true` if colors should be enabled.
///
/// This honors the [clicolors spec](http://bixense.com/clicolors/).
///
/// * `CLICOLOR != 0`: ANSI colors are supported and should be used when the program isn't piped.
/// * `CLICOLOR == 0`: Don't output ANSI color escape codes.
/// * `CLICOLOR_FORCE != 0`: ANSI colors should be enabled no matter what.
pub fn colors_enabled() -> bool {
    ENABLE_COLORS.load(Ordering::Relaxed)
}

/// Forces colorization on or off.
///
/// This overrides the default for the current process and changes the return value of the
/// `colors_enabled` function.
pub fn set_colors_enabled(val: bool) {
    ENABLE_COLORS.store(val, Ordering::Relaxed)
}

/// Configures the terminal for ANSI color support.
///
/// This is not needed on UNIX systems and normally automatically happens on windows
/// the first time `colors_enabled()` is called.  This automatic behavior however
/// can be disabled by removing the `terminal_autoconfig` feature flag.
///
/// When this function is called and the terminal was reconfigured, changes from
/// `set_colors_enabled` are reverted.
///
/// It returns `true` if the terminal supports colors after configuration or
/// `false` if not.
pub fn configure_terminal() -> bool {
    #[cfg(windows)]
    {
        if enable_ansi_mode() {
            /// if the terminal is configured we override the cached colors value
            /// with the default as otherwise we might have a wrong value.
            set_colors_enabled(enable_colors_by_default());
            true
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}
