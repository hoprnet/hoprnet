use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use utils;

#[derive(Debug, PartialEq)]
pub struct OSRelease {
    pub distro: Option<String>,
    pub version: Option<String>,
}

fn read_file(filename: &str) -> Result<String, Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn retrieve() -> Option<OSRelease> {
    if utils::file_exists("/etc/os-release") {
        if let Ok(release) = read_file("/etc/os-release") {
            Some(parse(release))
        } else {
            None
        }
    } else {
        if let Ok(release) = read_file("/usr/lib/os-release") {
            Some(parse(release))
        } else {
            None
        }
    }
}

pub fn parse(file: String) -> OSRelease {
    let distrib_regex = Regex::new(r#"NAME="(\w+)"#).unwrap();
    let version_regex = Regex::new(r#"VERSION_ID="?([\w\.]+)"#).unwrap();

    let distro = match distrib_regex.captures_iter(&file).next() {
        Some(m) => match m.get(1) {
            Some(distro) => Some(distro.as_str().to_owned()),
            None => None,
        },
        None => None,
    };

    let version = match version_regex.captures_iter(&file).next() {
        Some(m) => match m.get(1) {
            Some(version) => Some(version.as_str().to_owned()),
            None => None,
        },
        None => None,
    };

    OSRelease { distro, version }
}

mod tests {
    use super::*;

    #[test]
    fn parse_ubuntu_18_04_os_release() {
        let sample = "\
        NAME=\"Ubuntu\"\
        VERSION=\"18.04 LTS (Bionic Beaver)\"
        ID=ubuntu
        ID_LIKE=debian
        PRETTY_NAME=\"Ubuntu 18.04 LTS\"\
        VERSION_ID=\"18.04\"\
        HOME_URL=\"https://www.ubuntu.com/\"\
        SUPPORT_URL=\"https://help.ubuntu.com/\"\
        BUG_REPORT_URL=\"https://bugs.launchpad.net/ubuntu\"\
        PRIVACY_POLICY_URL=\"https://www.ubuntu.com/legal/terms-and-policies/privacy-policy\"\
        VERSION_CODENAME=bionic
        UBUNTU_CODENAME=bionic
        "
        .to_string();

        assert_eq!(
            parse(sample),
            OSRelease {
                distro: Some("Ubuntu".to_string()),
                version: Some("18.04".to_string()),
            }
        );
    }

    #[test]
    fn parse_alpine_3_9_5_os_release() {
        let sample = "\
        NAME=\"Alpine Linux\"
        ID=alpine
        VERSION_ID=3.9.5
        PRETTY_NAME=\"Alpine Linux v3.9\"
        HOME_URL=\"https://alpinelinux.org/\"
        BUG_REPORT_URL=\"https://bugs.alpinelinux.org/\"
        "
        .to_string();

        assert_eq!(
            parse(sample),
            OSRelease {
                distro: Some("Alpine".to_string()),
                version: Some("3.9.5".to_string()),
            }
        );
    }

    #[test]
    fn parse_deepin_20_3_os_release() {
        let sample = "\
        PRETTY_NAME=\"Deepin 20.3\"
        NAME=\"Deepin\"
        VERSION_ID=\"20.3\"
        VERSION=\"20.3\"
        ID=Deepin
        HOME_URL=\"https://www.deepin.org/\"
        "
        .to_string();

        assert_eq!(
            parse(sample),
            OSRelease {
                distro: Some("Deepin".to_string()),
                version: Some("20.3".to_string()),
            }
        );
    }

    #[test]
    fn parse_nixos_21_11_os_release() {
        let sample = "\
        NAME=NixOS
        ID=nixos
        VERSION=\"21.11 (Porcupine)\"
        VERSION_CODENAME=porcupine
        VERSION_ID=\"21.11\"
        BUILD_ID=\"21.11.20220325.d89f18a\"
        PRETTY_NAME=\"NixOS 21.11 (Porcupine)\"
        LOGO=\"nix-snowflake\"
        HOME_URL=\"https://nixos.org/\"
        DOCUMENTATION_URL=\"https://nixos.org/learn.html\"
        SUPPORT_URL=\"https://nixos.org/community.html\"
        BUG_REPORT_URL=\"https://github.com/NixOS/nixpkgs/issues\"
        "
        .to_string();

        assert_eq!(
            parse(sample),
            OSRelease {
                distro: Some("NixOS".to_string()),
                version: Some("21.11".to_string()),
            }
        );
    }
    #[test]
    fn parse_kali_2021_4_os_release() {
        let sample = "\
        PRETTY_NAME=\"Kali Linux GNU/Linux Rolling\"
        NAME=\"Kali\"
        ID=kali
        VERSION=\"2021.4\"
        VERSION_ID=\"2021.4\"
        VERSION_CODENAME=\"kali-rolling\"
        ID_LIKE=debian
        HOME_URL=\"https://www.kali.org/\"
        SUPPORT_URL=\"https://forums.kali.org/\"
        BUG_REPORT_URL=\"https://bugs.kali.org\"
        "
        .to_string();

        assert_eq!(
            parse(sample),
            OSRelease {
                distro: Some("Kali".to_string()),
                version: Some("2021.4".to_string()),
            }
        );
    }
}
