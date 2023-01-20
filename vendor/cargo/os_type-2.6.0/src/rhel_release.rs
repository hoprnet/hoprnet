use regex::Regex;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use utils;

pub struct RHELRelease {
    pub distro: Option<String>,
    pub version: Option<String>
}

fn read_file(filename: &str) -> Result<String, Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn retrieve() -> Option<RHELRelease> {
    if utils::file_exists("/etc/redhat-release") {
        if let Ok(release) = read_file("/etc/redhat-release") {
            Some(parse(release))
        } else {
            None
        }
    } else {
        if let Ok(release) = read_file("/etc/centos-release") {
            Some(parse(release))
        } else {
            None
        }
    }
}

pub fn parse(file: String) -> RHELRelease {
    let distrib_regex = Regex::new(r"(\w+) Linux release").unwrap();
    let version_regex = Regex::new(r"release\s([\w\.]+)").unwrap();

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

    let version = match version_regex.captures_iter(&file).next() {
        Some(m) => {
            match m.get(1) {
                Some(version) => {
                    Some(version.as_str().to_owned())
                },
                None => None
            }
        },
        None => None
    };

    RHELRelease {
        distro: distro,
        version: version
    }
}
