use regex::Regex;
use std::process::Command;

pub struct WindowsVer {
    pub version: Option<String>
}

pub fn retrieve() -> Option<WindowsVer> {
    let output = match Command::new("ver").output() {
        Ok(o) => o,
        Err(_) => return None
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse(stdout.to_string()))
}

pub fn parse(output: String) -> WindowsVer {
    let version_regex = Regex::new(r"^Microsoft Windows \[Version\s(\d+\.\d+\.\d+)\]$").unwrap();

    let version = match version_regex.captures_iter(&output).next() {
        Some(m) => {
            match m.get(1) {
                Some(version) => Some(version.as_str().to_owned()),
                None => None
            }
        },
        None => None
    };
    WindowsVer { version: version }
}
