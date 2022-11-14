extern crate regex;
#[path = "../src/lsb_release.rs"]
mod lsb_release;
#[path = "../src/utils.rs"]
mod utils;

fn file() -> String {
    "
Distributor ID:	Debian
Description:	Debian GNU/Linux 7.8 (wheezy)
Release:	7.8
Codename:	wheezy
"
    .to_string()
}

fn arch_file() -> String {
    "
LSB Version:	1.4
Distributor ID:	Arch
Description:	Arch Linux
Release:	rolling
Codename:	n/a
"
    .to_string()
}

fn manjaro_file() -> String {
    "
LSB Version:	n/a
Distributor ID:	ManjaroLinux
Description:	Manjaro Linux
Release:	17.1.7
Codename:	Hakoila
"
    .to_string()
}

fn opensuse_tumbleweed_file() -> String {
    "
LSB Version:    n/a
Distributor ID: openSUSE
Description:    openSUSE Tumbleweed
Release:        20170712
Codename:       n/a
"
    .to_string()
}

fn nixos_file() -> String {
    "
    No LSB modules are available.
    Distributor ID:	NixOS
    Description:	NixOS 21.11 (Porcupine)
    Release:	21.11
    Codename:	porcupine
    "
    .to_string()
}

#[test]
pub fn test_parses_lsb_distro() {
    let parse_results = lsb_release::parse(file());
    assert_eq!(parse_results.distro, Some("Debian".to_string()));
}

#[test]
pub fn test_parses_lsb_version() {
    let parse_results = lsb_release::parse(file());
    assert_eq!(parse_results.version, Some("7.8".to_string()));
}

#[test]
pub fn test_parses_arch_lsb_distro() {
    let parse_results = lsb_release::parse(arch_file());
    assert_eq!(parse_results.distro, Some("Arch".to_string()));
}

#[test]
pub fn test_parses_arch_lsb_version() {
    let parse_results = lsb_release::parse(arch_file());
    assert_eq!(parse_results.version, Some("rolling".to_string()));
}

#[test]
pub fn test_parses_manjaro_lsb_distro() {
    let parse_results = lsb_release::parse(manjaro_file());
    assert_eq!(parse_results.distro, Some("ManjaroLinux".to_string()));
}

#[test]
pub fn test_parses_manjaro_lsb_version() {
    let parse_results = lsb_release::parse(manjaro_file());
    assert_eq!(parse_results.version, Some("17.1.7".to_string()));
}

#[test]
pub fn test_parses_opensuse_lsb_distro() {
    let parse_results = lsb_release::parse(opensuse_tumbleweed_file());
    assert_eq!(parse_results.distro, Some("openSUSE".to_string()));
}

#[test]
pub fn test_parses_opensuse_lsb_version() {
    let parse_results = lsb_release::parse(opensuse_tumbleweed_file());
    assert_eq!(parse_results.version, Some("20170712".to_string()));
}

#[test]
pub fn test_parses_nixos_lsb_distro() {
    let parse_results = lsb_release::parse(nixos_file());
    assert_eq!(parse_results.distro, Some("NixOS".to_string()));
}

#[test]
pub fn test_parses_nixos_lsb_version() {
    let parse_results = lsb_release::parse(nixos_file());
    assert_eq!(parse_results.version, Some("21.11".to_string()));
}
