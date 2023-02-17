/// Output stream to [`get()`][crate::get] the [`Color`][crate::Color] state for
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
    /// When unsure which will be used (lowest common denominator of `Stdout` and `Stderr`)
    Either,
}
