use crate::error::Result;

#[cfg(feature = "wasm")]
// These functions are meant to be used in pure Rust code, since they are cleared from WASM types
pub fn coerce_version(version: &str) -> Result<String> {
    wasm::coerce_version(version).map_err(crate::error::RealError::from)
}

#[cfg(not(feature = "wasm"))]
// These functions are meant to be used in pure Rust code, since they are cleared from WASM types
pub fn coerce_version(_version: &str) -> Result<String> {
    todo!("Implement native coerce_version")
}

#[cfg(feature = "wasm")]
pub fn satisfies(version: &str, range: &str) -> Result<bool> {
    wasm::satisfies(version, range).map_err(crate::error::RealError::from)
}

#[cfg(not(feature = "wasm"))]
pub fn satisfies(_version: &str, _range: &str) -> Result<bool> {
    todo!("Implement native satisfies")
}

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "@hoprnet/hopr-real")]
    extern "C" {
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
