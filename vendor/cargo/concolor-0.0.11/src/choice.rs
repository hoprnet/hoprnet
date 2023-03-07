/// Selection for overriding color output with [`set`][crate::set]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ColorChoice {
    Auto,
    AlwaysAnsi,
    Always,
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}
