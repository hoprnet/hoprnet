use crate::builder::PossibleValue;
use crate::derive::ValueEnum;

/// Represents the color preferences for program output
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ColorChoice {
    /// Enables colored output only when the output is going to a terminal or TTY.
    ///
    /// **NOTE:** This is the default behavior of `clap`.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms).
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Auto)
    ///     .get_matches();
    /// ```
    Auto,

    /// Enables colored output regardless of whether or not the output is going to a terminal/TTY.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms).
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Always)
    ///     .get_matches();
    /// ```
    Always,

    /// Disables colored output no matter if the output is going to a terminal/TTY, or not.
    ///
    /// # Platform Specific
    ///
    /// This setting only applies to Unix, Linux, and macOS (i.e. non-Windows platforms)
    ///
    /// # Examples
    ///
    #[cfg_attr(not(feature = "color"), doc = " ```ignore")]
    #[cfg_attr(feature = "color", doc = " ```no_run")]
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Never)
    ///     .get_matches();
    /// ```
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl std::str::FromStr for ColorChoice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}

impl ValueEnum for ColorChoice {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Auto, Self::Always, Self::Never]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Auto => {
                PossibleValue::new("auto").help("Use colored output if writing to a terminal/TTY")
            }
            Self::Always => PossibleValue::new("always").help("Always use colored output"),
            Self::Never => PossibleValue::new("never").help("Never use colored output"),
        })
    }
}
