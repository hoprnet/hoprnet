use log::trace;

use crate::{Bitness, Info, Type, Version};

pub fn current_platform() -> Info {
    trace!("android::current_platform is called");

    let info = Info {
        os_type: Type::Android,
        version: Version::unknown(),
        bitness: Bitness::Unknown,
    };
    trace!("Returning {:?}", info);
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::Android, version.os_type());
    }
}
