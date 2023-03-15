extern crate regex;
#[path="../src/rhel_release.rs"]
mod rhel_release;
#[path="../src/utils.rs"]
mod utils;

fn file() -> String {
     "CentOS Linux release 7.3.1611 (Core)".into()
}

#[test]
pub fn test_parses_distribution() {
    let parse_results = rhel_release::parse(file());
    assert_eq!(parse_results.distro, Some("CentOS".to_string()));
}

#[test]
pub fn test_parses_version() {
    let parse_results = rhel_release::parse(file());
    assert_eq!(parse_results.version, Some("7.3.1611".to_string()));
}