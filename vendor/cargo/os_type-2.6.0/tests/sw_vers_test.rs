extern crate regex;
#[path="../src/sw_vers.rs"]
mod sw_vers;
#[path="../src/utils.rs"]
mod utils;

fn file() -> String {
"
ProductName:	Mac OS X
ProductVersion:	10.10.5
BuildVersion:	14F27
".to_string()
}

#[test]
pub fn parses_product_name() {
    let info = sw_vers::parse(file());
    assert_eq!(info.product_name, Some("Mac OS X".to_string()));
}

#[test]
pub fn parses_product_version() {
    let info = sw_vers::parse(file());
    assert_eq!(info.product_version, Some("10.10.5".to_string()));
}

#[test]
pub fn parses_build_version() {
    let info = sw_vers::parse(file());
    assert_eq!(info.build_version, Some("14F27".to_string()));
}
