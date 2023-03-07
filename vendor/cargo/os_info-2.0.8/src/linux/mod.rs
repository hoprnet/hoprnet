mod file_release;
mod lsb_release;

use log::trace;

use crate::{bitness, Bitness, Info, Type, Version};

pub fn current_platform() -> Info {
    trace!("linux::current_platform is called");

    let mut info = lsb_release::get()
        .or_else(file_release::get)
        .unwrap_or_else(|| Info::new(Type::Linux, Version::unknown(), Bitness::Unknown));
    info.bitness = bitness::get();

    trace!("Returning {:?}", info);
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_type() {
        let version = current_platform();
        match version.os_type() {
            Type::Alpine
            | Type::Amazon
            | Type::Arch
            | Type::Centos
            | Type::Debian
            | Type::EndeavourOS
            | Type::Fedora
            | Type::Linux
            | Type::Manjaro
            | Type::openSUSE
            | Type::OracleLinux
            | Type::Pop
            | Type::Redhat
            | Type::RedHatEnterprise
            | Type::Solus
            | Type::SUSE
            | Type::Ubuntu => (),
            os_type => {
                panic!("Unexpected OS type: {}", os_type);
            }
        }
    }
}
