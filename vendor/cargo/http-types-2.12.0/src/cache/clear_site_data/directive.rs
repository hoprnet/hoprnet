use std::fmt::Display;
use std::str::FromStr;

/// An HTTP `Clear-Site-Data` directive.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Clear-Site-Data#Directives)
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ClearDirective {
    /// Indicates that the server wishes to remove locally cached data (i.e. the
    /// browser cache, see HTTP caching) for the origin of the response URL.
    /// Depending on the browser, this might also clear out things like
    /// pre-rendered pages, script caches, WebGL shader caches, or address bar
    /// suggestions.
    Cache,
    /// Indicates that the server wishes to remove all cookies for the origin of
    /// the response URL. HTTP authentication credentials are also cleared out.
    /// This affects the entire registered domain, including subdomains. So
    /// https://example.com as well as https://stage.example.com, will have
    /// cookies cleared.
    Cookies,
    /// Indicates that the server wishes to remove all DOM storage for the origin
    /// of the response URL.
    Storage,
    /// Indicates that the server wishes to reload all browsing contexts for the
    /// origin of the response (Location.reload).
    ExecutionContexts,
}

impl ClearDirective {
    /// Get the formatted string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ClearDirective::Cache => r#""cache""#,
            ClearDirective::Cookies => r#""cookies""#,
            ClearDirective::Storage => r#""storage""#,
            ClearDirective::ExecutionContexts => r#""executionContexts""#,
        }
    }
}

impl Display for ClearDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ClearDirective {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            r#""cache""# => Ok(Self::Cache),
            r#""cookies""# => Ok(Self::Cookies),
            r#""storage""# => Ok(Self::Storage),
            r#""executionContexts""# => Ok(Self::ExecutionContexts),
            _ => todo!(),
        }
    }
}
