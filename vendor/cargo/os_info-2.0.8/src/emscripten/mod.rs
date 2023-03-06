use log::trace;

use crate::{Bitness, Info, Type, Version};

// TODO: Somehow get the real OS version?
pub fn current_platform() -> Info {
    trace!("emscripten::current_platform is called");

    let info = Info {
        os_type: Type::Emscripten,
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
        assert_eq!(Type::Emscripten, version.os_type());
    }
}
