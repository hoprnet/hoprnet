use regex::Regex;
use std::process::Command;

pub struct LsbRelease {
    pub distro: Option<String>,
    pub version: Option<String>
}

pub fn retrieve() -> Option<LsbRelease> {
    let output = match Command::new("lsb_release").arg("-a").output() {
        Ok(o)  => o,
        Err(_) =>return None
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse(stdout.to_string()))
}

pub fn is_available() -> bool {
    match Command::new("lsb_release").output() {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn parse(file: String) -> LsbRelease {
    let distrib_regex = Regex::new(r"Distributor ID:\s*(\w+)").unwrap();
    let distrib_release_regex = Regex::new(r"Release:\s*([\w\.]+)").unwrap();

    let distro = match distrib_regex.captures_iter(&file).next() {
        Some(m) => {
            match m.get(1) {
                Some(distro) => {
                    Some(distro.as_str().to_owned())
                },
                None => None
            }
        },
        None => None
    };

    let version = match distrib_release_regex.captures_iter(&file).next() {
        Some(m) => {
            match m.get(1) {
                Some(version) => Some(version.as_str().to_owned()),
                None => None
            }
        },
        None => None
    };

    LsbRelease {
        distro: distro,
        version: version
    }
}
