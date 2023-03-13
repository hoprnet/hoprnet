// spell-checker:ignore itertools, iproduct, bitnesses

use std::fmt::{self, Display, Formatter};

use super::{Bitness, Type, Version};

/// Holds information about operating system (type, version, etc.).
///
/// The best way to get string representation of the operation system information is to use its
/// `Display` implementation.
///
/// # Examples
///
/// ```
/// use os_info;
///
/// let info = os_info::get();
/// println!("OS information: {}", info);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    /// Operating system type. See `Type` for details.
    pub(crate) os_type: Type,
    /// Operating system version. See `Version` for details.
    pub(crate) version: Version,
    /// Operating system architecture in terms of how many bits compose the basic values it can deal
    /// with. See `Bitness` for details.
    pub(crate) bitness: Bitness,
}

impl Info {
    /// Constructs a new `Info` instance with unknown type, version and bitness.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_info::{Info, Type, Version, Bitness};
    ///
    /// let info = Info::unknown();
    /// assert_eq!(Type::Unknown, info.os_type());
    /// assert_eq!(Version::unknown(), *info.version());
    /// assert_eq!(Bitness::Unknown, info.bitness());
    /// ```
    pub fn unknown() -> Self {
        Self {
            os_type: Type::Unknown,
            version: Version::unknown(),
            bitness: Bitness::Unknown,
        }
    }

    /// Constructs a new `Info` instance with the given type, version and bitness.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_info::{Info, Type, Version, Bitness};
    ///
    /// let os_type = Type::Unknown;
    /// let version = Version::unknown();
    /// let bitness = Bitness::Unknown;
    /// let info = Info::new(os_type, version.clone(), bitness);
    /// assert_eq!(os_type, info.os_type());
    /// assert_eq!(version, *info.version());
    /// assert_eq!(bitness, info.bitness());
    /// ```
    pub fn new(os_type: Type, version: Version, bitness: Bitness) -> Self {
        Self {
            os_type,
            version,
            bitness,
        }
    }

    /// Returns operating system type. See `Type` for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_info::{Info, Type};
    ///
    /// let info = Info::unknown();
    /// assert_eq!(Type::Unknown, info.os_type());
    /// ```
    pub fn os_type(&self) -> Type {
        self.os_type
    }

    /// Returns operating system version. See `Version` for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_info::{Info, Version};
    ///
    /// let info = Info::unknown();
    /// assert_eq!(Version::unknown(), *info.version());
    /// ```
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns operating system bitness. See `Bitness` for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_info::{Info, Bitness};
    ///
    /// let info = Info::unknown();
    /// assert_eq!(Bitness::Unknown, info.bitness());
    /// ```
    pub fn bitness(&self) -> Bitness {
        self.bitness
    }
}

impl Default for Info {
    fn default() -> Self {
        Self::unknown()
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.os_type)?;
        write!(f, " ({})", self.version)?;
        write!(f, " ({})", self.bitness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::iproduct;
    use pretty_assertions::assert_eq;

    #[test]
    fn unknown() {
        let info = Info::unknown();
        assert_eq!(Type::Unknown, info.os_type());
        assert_eq!(&Version::unknown(), info.version());
        assert_eq!(Bitness::Unknown, info.bitness());
    }

    #[test]
    fn new() {
        let types = [
            Type::Unknown,
            Type::Android,
            Type::Emscripten,
            Type::Linux,
            Type::Redhat,
            Type::Ubuntu,
            Type::Debian,
            Type::Arch,
            Type::Centos,
            Type::Fedora,
            Type::Solus,
            Type::EndeavourOS,
            Type::Manjaro,
            Type::Alpine,
            Type::Macos,
            Type::Redox,
            Type::Windows,
        ];

        let versions = [
            Version::unknown(),
            Version::semantic(0, 0, 0, None),
            Version::semantic(1, 2, 3, Some("e".to_owned())),
            Version::custom("version".to_owned(), None),
            Version::custom("different version".to_owned(), Some("edition".to_owned())),
        ];

        let bitnesses = [Bitness::Unknown, Bitness::X32, Bitness::X64];

        for (os_type, version, bitness) in iproduct!(&types, &versions, &bitnesses) {
            let info = Info::new(*os_type, version.clone(), *bitness);
            assert_eq!(*os_type, info.os_type());
            assert_eq!(version, info.version());
        }
    }
}
