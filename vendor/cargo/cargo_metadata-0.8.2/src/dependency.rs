//! This module contains `Dependency` and the types/functions it uses for deserialization.

use semver::VersionReq;
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
/// Dependencies can come in three kinds
pub enum DependencyKind {
    #[serde(rename = "normal")]
    /// The 'normal' kind
    Normal,
    #[serde(rename = "dev")]
    /// Those used in tests only
    Development,
    #[serde(rename = "build")]
    /// Those used in build scripts only
    Build,
    #[doc(hidden)]
    #[serde(other)]
    Unknown,
}

impl Default for DependencyKind {
    fn default() -> DependencyKind {
        DependencyKind::Normal
    }
}

/// The `kind` can be `null`, which is interpreted as the default - `Normal`.
fn parse_dependency_kind<'de, D>(d: D) -> Result<DependencyKind, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A dependency of the main crate
pub struct Dependency {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// The source of dependency
    pub source: Option<String>,
    /// The required version
    pub req: VersionReq,
    /// The kind of dependency this is
    #[serde(deserialize_with = "parse_dependency_kind")]
    pub kind: DependencyKind,
    /// Whether this dependency is required or optional
    pub optional: bool,
    /// Whether the default features in this dependency are used.
    pub uses_default_features: bool,
    /// The list of features enabled for this dependency.
    pub features: Vec<String>,
    /// The target this dependency is specific to.
    ///
    /// Use the [`Display`] trait to access the contents.
    ///
    /// [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
    pub target: Option<Platform>,
    /// If the dependency is renamed, this is the new name for the dependency
    /// as a string.  None if it is not renamed.
    pub rename: Option<String>,
    /// The URL of the index of the registry where this dependency is from.
    ///
    /// If None, the dependency is from crates.io.
    pub registry: Option<String>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

/// A target platform.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Platform(String);

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
