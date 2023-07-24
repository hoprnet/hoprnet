//! Extension traits for `str` primitive type.

/// Alternative for `std::str::pattern::Pattern` that does not require
/// nightly Rust (as `Pattern` is unstable yet).
#[doc(hidden)]
pub enum AltPattern<'a> {
    Str(&'a str),
    Char(char),
}

impl<'a> From<&'a str> for AltPattern<'a> {
    fn from(data: &'a str) -> Self {
        Self::Str(data)
    }
}

impl<'a> From<char> for AltPattern<'a> {
    fn from(data: char) -> Self {
        Self::Char(data)
    }
}

/// Extension trait with useful methods for primitive type [`str`].
///
/// [`str`]: https://doc.rust-lang.org/std/primitive.str.html
pub trait StrExt {
    /// Version of [`str::splitn`] which expects the **exact**
    /// amount of entries obtained after splitting. This method
    /// returns `Vec`, as [`SplitN`] iterator depends on the unstable
    /// type [`Pattern`] and cannot be returned on the stable Rust.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdext::prelude::*;
    ///
    /// let data = "Hello world";
    /// let splitted = data.splitn_exact(2, " ").unwrap();
    /// assert_eq!(&splitted, &["Hello", "world"]);
    ///
    /// let splitted = data.splitn_exact(5, '-');
    /// assert!(splitted.is_none());
    /// ```
    ///
    /// [`str::splitn`]: https://doc.rust-lang.org/std/primitive.str.html#method.splitn
    /// [`SplitN`]: https://doc.rust-lang.org/std/str/struct.SplitN.html
    /// [`Pattern`]: https://doc.rust-lang.org/std/str/pattern/trait.Pattern.html
    fn splitn_exact<'a, P: Into<AltPattern<'a>>>(
        &'a self,
        n: usize,
        pat: P,
    ) -> Option<Vec<&'a str>>;
}

impl StrExt for &str {
    fn splitn_exact<'a, P: Into<AltPattern<'a>>>(
        &'a self,
        n: usize,
        pat: P,
    ) -> Option<Vec<&'a str>> {
        let pat = pat.into();
        // Overcome for `&str` splitting API: it accepts generic arguments as separators,
        // but the `Pattern` trait is unstable, thus it's impossible to just forward arguments
        // inside on stable Rust.
        let splitted: Vec<_> = match pat {
            AltPattern::Str(sep) => self.splitn(n, sep).collect(),
            AltPattern::Char(sep) => self.splitn(n, sep).collect(),
        };

        if splitted.len() == n {
            Some(splitted)
        } else {
            None
        }
    }
}
