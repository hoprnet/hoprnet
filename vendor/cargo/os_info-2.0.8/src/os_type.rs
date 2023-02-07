use std::fmt::{self, Display, Formatter};

/// A list of supported operating system types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum Type {
    /// Alpine Linux (<https://en.wikipedia.org/wiki/Alpine_Linux>).
    Alpine,
    /// Amazon Linux AMI (<https://en.wikipedia.org/wiki/Amazon_Machine_Image#Amazon_Linux_AMI>).
    Amazon,
    /// Android (<https://en.wikipedia.org/wiki/Android_(operating_system)>).
    Android,
    /// Arch Linux (<https://en.wikipedia.org/wiki/Arch_Linux>).
    Arch,
    /// CentOS (<https://en.wikipedia.org/wiki/CentOS>).
    Centos,
    /// Debian (<https://en.wikipedia.org/wiki/Debian>).
    Debian,
    /// Emscripten (<https://en.wikipedia.org/wiki/Emscripten>).
    Emscripten,
    /// EndeavourOS (<https://en.wikipedia.org/wiki/EndeavourOS>).
    EndeavourOS,
    /// Fedora (<https://en.wikipedia.org/wiki/Fedora_(operating_system)>).
    Fedora,
    /// Linux based operating system (<https://en.wikipedia.org/wiki/Linux>).
    Linux,
    /// Mac OS X/OS X/macOS (<https://en.wikipedia.org/wiki/MacOS>).
    Macos,
    /// Manjaro (<https://en.wikipedia.org/wiki/Manjaro>).
    Manjaro,
    /// openSUSE (<https://en.wikipedia.org/wiki/OpenSUSE>).
    openSUSE,
    /// Oracle Linux (<https://en.wikipedia.org/wiki/Oracle_Linux>).
    OracleLinux,
    /// Pop!_OS (<https://en.wikipedia.org/wiki/Pop!_OS>)
    Pop,
    /// Red Hat Linux (<https://en.wikipedia.org/wiki/Red_Hat_Linux>).
    Redhat,
    /// Red Hat Enterprise Linux (<https://en.wikipedia.org/wiki/Red_Hat_Enterprise_Linux>).
    RedHatEnterprise,
    /// Redox (<https://en.wikipedia.org/wiki/Redox_(operating_system)>).
    Redox,
    /// Solus (<https://en.wikipedia.org/wiki/Solus_(operating_system)>).
    Solus,
    /// SUSE Linux Enterprise Server (<https://en.wikipedia.org/wiki/SUSE_Linux_Enterprise>).
    SUSE,
    /// Ubuntu (<https://en.wikipedia.org/wiki/Ubuntu_(operating_system)>).
    Ubuntu,
    /// Unknown operating system.
    Unknown,
    /// Windows (<https://en.wikipedia.org/wiki/Microsoft_Windows>).
    Windows,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Type::Redhat => write!(f, "Red Hat Linux"),
            Type::Arch => write!(f, "Arch Linux"),
            Type::Centos => write!(f, "CentOS"),
            Type::Macos => write!(f, "Mac OS"),
            _ => write!(f, "{:?}", self),
        }
    }
}
