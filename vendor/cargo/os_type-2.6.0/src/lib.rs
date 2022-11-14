extern crate regex;

use std::process::Command;
mod lsb_release;
mod os_release;
mod rhel_release;
mod sw_vers;
mod utils;
mod windows_ver;

///A list of supported operating system types
#[derive(Debug, PartialEq, Clone)]
pub enum OSType {
    Unknown,
    Redhat,
    OSX,
    Ubuntu,
    Debian,
    Arch,
    Manjaro,
    CentOS,
    OpenSUSE,
    Alpine,
    Deepin,
    NixOS,
    Kali,
}

/// Holds information about Operating System type and its version
/// If the version could not be fetched it defaults to `0.0.0`
#[derive(Debug, Clone, PartialEq)]
pub struct OSInformation {
    pub os_type: self::OSType,
    pub version: String,
}

fn default_version() -> String {
    "0.0.0".into()
}

fn unknown_os() -> OSInformation {
    OSInformation {
        os_type: OSType::Unknown,
        version: default_version(),
    }
}

fn is_os_x() -> bool {
    match Command::new("sw_vers").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn get_sw_vers() -> OSInformation {
    if let Some(osx_info) = sw_vers::retrieve() {
        OSInformation {
            os_type: OSType::OSX,
            version: osx_info.product_version.unwrap_or(default_version()),
        }
    } else {
        unknown_os()
    }
}

fn lsb_release() -> OSInformation {
    match lsb_release::retrieve() {
        Some(release) => {
            if release.distro == Some("Ubuntu".to_string()) {
                OSInformation {
                    os_type: OSType::Ubuntu,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("Debian".to_string()) {
                OSInformation {
                    os_type: OSType::Debian,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("Arch".to_string()) {
                OSInformation {
                    os_type: OSType::Arch,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("ManjaroLinux".to_string()) {
                OSInformation {
                    os_type: OSType::Manjaro,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("CentOS".to_string()) {
                OSInformation {
                    os_type: OSType::CentOS,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("openSUSE".to_string()) {
                OSInformation {
                    os_type: OSType::OpenSUSE,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("NixOS".to_string()) {
                OSInformation {
                    os_type: OSType::NixOS,
                    version: release.version.unwrap_or(default_version()),
                }
            } else if release.distro == Some("Kali".to_string()) {
                OSInformation {
                    os_type: OSType::Kali,
                    version: release.version.unwrap_or(default_version()),
                }
            } else {
                unknown_os()
            }
        }
        None => unknown_os(),
    }
}

fn rhel_release() -> OSInformation {
    match rhel_release::retrieve() {
        Some(release) => {
            if release.distro == Some("CentOS".to_string()) {
                OSInformation {
                    os_type: OSType::CentOS,
                    version: release.version.unwrap_or(default_version()),
                }
            } else {
                OSInformation {
                    os_type: OSType::Redhat,
                    version: release.version.unwrap_or(default_version()),
                }
            }
        }
        None => unknown_os(),
    }
}

fn os_release() -> OSInformation {
    match os_release::retrieve() {
        Some(release) => match release.distro {
            Some(distro) => {
                if distro.starts_with("Ubuntu") {
                    OSInformation {
                        os_type: OSType::Ubuntu,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("Debian") {
                    OSInformation {
                        os_type: OSType::Debian,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("Arch") {
                    OSInformation {
                        os_type: OSType::Arch,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("CentOS") {
                    OSInformation {
                        os_type: OSType::CentOS,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("openSUSE") {
                    OSInformation {
                        os_type: OSType::OpenSUSE,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("Alpine") {
                    OSInformation {
                        os_type: OSType::Alpine,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("Deepin") {
                    OSInformation {
                        os_type: OSType::Deepin,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("NixOS") {
                    OSInformation {
                        os_type: OSType::NixOS,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else if distro.starts_with("Kali") {
                    OSInformation {
                        os_type: OSType::Kali,
                        version: release.version.unwrap_or(default_version()),
                    }
                } else {
                    unknown_os()
                }
            }
            None => unknown_os(),
        },
        None => unknown_os(),
    }
}

///Returns the current operating system type
///
///#Example
///
///```
///use os_type;
///let os = os_type::current_platform();
///println!("Type: {:?}", os.os_type);
///println!("Version: {}", os.version);
///```
pub fn current_platform() -> OSInformation {
    if is_os_x() {
        get_sw_vers()
    } else if lsb_release::is_available() {
        lsb_release()
    } else if utils::file_exists("/etc/os-release") {
        os_release()
    } else if utils::file_exists("/etc/redhat-release") || utils::file_exists("/etc/centos-release")
    {
        rhel_release()
    } else {
        unknown_os()
    }
}
