use std::{fs::File, io::Read, path::Path};

use log::{trace, warn};

use crate::{matcher::Matcher, Bitness, Info, Type, Version, VersionType};

pub fn get() -> Option<Info> {
    retrieve(&DISTRIBUTIONS)
}

fn retrieve(distributions: &[ReleaseInfo]) -> Option<Info> {
    for release_info in distributions {
        if !Path::new(release_info.path).exists() {
            trace!("Path '{}' doesn't exist", release_info.path);
            continue;
        }

        let mut file = match File::open(&release_info.path) {
            Ok(val) => val,
            Err(e) => {
                warn!("Unable to open {:?} file: {:?}", release_info.path, e);
                continue;
            }
        };

        let mut file_content = String::new();
        if let Err(e) = file.read_to_string(&mut file_content) {
            warn!("Unable to read {:?} file: {:?}", release_info.path, e);
            continue;
        }

        let os_type = Matcher::KeyValue { key: "NAME" }
            .find(&file_content)
            .and_then(|name| get_type(&name))
            .unwrap_or(release_info.os_type);

        let version = release_info
            .version_matcher
            .find(&file_content)
            .map(|x| Version::new(VersionType::from_string(&x), None))
            .unwrap_or_else(Version::unknown);

        return Some(Info::new(os_type, version, Bitness::Unknown));
    }

    None
}

fn get_type(name: &str) -> Option<Type> {
    match name.to_lowercase().as_ref() {
        "arch linux" => Some(Type::Arch),
        "centos linux" => Some(Type::Centos),
        "ubuntu" => Some(Type::Ubuntu),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct ReleaseInfo<'a> {
    os_type: Type,
    path: &'a str,
    version_matcher: Matcher,
}

/// List of all supported distributions and the information on how to parse their version from the
/// release file.
const DISTRIBUTIONS: [ReleaseInfo; 5] = [
    // Due to shenanigans with Oracle Linux including an /etc/redhat-release file that states
    // that the OS is Red Hat Enterprise Linux, this /etc/os-release file MUST be checked
    // before this code checks /etc/redhat-release. If it does not get run first,
    // it will unintentionally report that the operating system is Red Hat Enterprise Linux
    // instead of Oracle Linux.
    ReleaseInfo {
        os_type: Type::OracleLinux,
        path: "/etc/os-release",
        version_matcher: Matcher::KeyValue { key: "VERSION_ID" },
    },
    ReleaseInfo {
        os_type: Type::Centos,
        path: "/etc/centos-release",
        version_matcher: Matcher::PrefixedVersion { prefix: "release" },
    },
    ReleaseInfo {
        os_type: Type::Fedora,
        path: "/etc/fedora-release",
        version_matcher: Matcher::PrefixedVersion { prefix: "release" },
    },
    ReleaseInfo {
        os_type: Type::Redhat,
        path: "/etc/redhat-release",
        version_matcher: Matcher::PrefixedVersion { prefix: "release" },
    },
    ReleaseInfo {
        os_type: Type::Alpine,
        path: "/etc/alpine-release",
        version_matcher: Matcher::AllTrimmed,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn oracle_linux() {
        let mut distributions = [DISTRIBUTIONS[0].clone()];
        distributions[0].path = "src/linux/tests/os-release";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::OracleLinux);
        assert_eq!(info.version, Version::semantic(8, 1, 0, None));
    }

    #[test]
    fn os_release_centos() {
        let mut distributions = [DISTRIBUTIONS[0].clone()];
        distributions[0].path = "src/linux/tests/os-release-centos";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Centos);
        assert_eq!(info.version, Version::semantic(7, 0, 0, None));
    }

    #[test]
    fn os_release_ubuntu() {
        let mut distributions = [DISTRIBUTIONS[0].clone()];
        distributions[0].path = "src/linux/tests/os-release-ubuntu";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Ubuntu);
        assert_eq!(info.version, Version::semantic(18, 10, 0, None));
    }

    #[test]
    fn centos() {
        let mut distributions = [DISTRIBUTIONS[1].clone()];
        distributions[0].path = "src/linux/tests/centos-release";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Centos);
        assert_eq!(info.version, Version::custom("XX", None));
    }

    #[test]
    fn fedora() {
        let mut distributions = [DISTRIBUTIONS[2].clone()];
        distributions[0].path = "src/linux/tests/fedora-release";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Fedora);
        assert_eq!(info.version, Version::semantic(26, 0, 0, None));
    }

    #[test]
    fn redhat() {
        let mut distributions = [DISTRIBUTIONS[3].clone()];
        distributions[0].path = "src/linux/tests/redhat-release";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Redhat);
        assert_eq!(info.version, Version::custom("XX", None));
    }

    #[test]
    fn alpine() {
        let mut distributions = [DISTRIBUTIONS[4].clone()];
        distributions[0].path = "src/linux/tests/alpine-release";

        let info = retrieve(&distributions).unwrap();
        assert_eq!(info.os_type(), Type::Alpine);
        assert_eq!(info.version, Version::custom("A.B.C", None));
    }
}
