use crate::error::{RealError, Result};

// These functions are meant to be used in pure Rust code, since they are cleared from WASM types

pub fn read_file(file: &str) -> Result<Box<[u8]>> {
    wasm::read_file(file).map_err(RealError::from)
}

pub fn write_file(file: &str, data: &[u8]) -> Result<()> {
    wasm::write_file(file, data).map_err(RealError::from)
}

pub fn coerce_version(version: &str) -> Result<String> {
    wasm::coerce_version(version).map_err(RealError::from)
}

pub fn satisfies(version: &str, range: &str) -> Result<bool> {
    wasm::satisfies(version, range).map_err(RealError::from)
}

mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "@hoprnet/hopr-real")]
    extern "C" {
        // Imported from `node:fs`

        /// Reads the given file and returns it as array of bytes.
        #[wasm_bindgen(catch)]
        pub fn read_file(file: &str) -> Result<Box<[u8]>, JsValue>;

        /// Writes given data to the given file.
        #[wasm_bindgen(catch)]
        pub fn write_file(file: &str, data: &[u8]) -> Result<(), JsValue>;

        // Imported from `semver`

        /// Coerce version string, e.g. `43.12.11-next.3` to `43.12.11`
        #[wasm_bindgen(catch)]
        pub fn coerce_version(version: &str) -> Result<String, JsValue>;

        /// Checks whether a version satisfies a version range according
        /// to Node.js' interpretation of semantic versioning.
        #[wasm_bindgen(catch)]
        pub fn satisfies(version: &str, range: &str) -> Result<bool, JsValue>;
    }
}
