use std::process::Command;
use std::str;

use log::{error, trace};

use crate::{bitness, uname::uname, Info, Type, Version};

pub fn current_platform() -> Info {
    trace!("freebsd::current_platform is called");

    let version = uname()
        .map(Version::from_string)
        .unwrap_or_else(|| Version::Unknown);

    let info = Info {
        os_type: get_os(),
        version,
        bitness: bitness::get(),
        ..Default::default()
    };

    trace!("Returning {:?}", info);
    info
}

fn get_os() -> Type {
    let os = Command::new("uname")
        .arg("-s")
        .output()
        .expect("Failed to get OS");

    match str::from_utf8(&os.stdout) {
        Ok("FreeBSD\n") => {
            let check_hardening = Command::new("sysctl")
                .arg("hardening.version")
                .output()
                .expect("Failed to check if is hardened");
            match str::from_utf8(&check_hardening.stderr) {
                Ok("0\n") => Type::HardenedBSD,
                Ok(_) => Type::FreeBSD,
                Err(_) => Type::FreeBSD,
            }
        }
        Ok("MidnightBSD\n") => Type::MidnightBSD,
        Ok(_) => Type::Unknown,
        Err(_) => Type::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::FreeBSD, version.os_type());
    }
}
