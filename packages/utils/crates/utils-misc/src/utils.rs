use real_base::error::RealError;
use real_base::error::RealError::GeneralError;
use serde::Deserialize;

#[cfg(any(not(feature = "wasm"), test))]
use real_base::file::native::read_file;
#[cfg(all(feature = "wasm", not(test)))]
use real_base::file::wasm::read_file;

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

/// Reads the given package.json file and determines its version.
pub fn get_package_version(package_file: &str) -> Result<String, RealError> {
    let file_data = read_file(package_file)?;

    match serde_json::from_slice::<PackageJsonFile>(file_data.as_ref()) {
        Ok(package_json) => Ok(package_json.version),
        Err(e) => Err(GeneralError(e.to_string())),
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::ok_or_jserr;
    use std::collections::HashMap;
    use wasm_bindgen::prelude::*;

    pub type JsResult<T> = Result<T, JsValue>;

    /// Helper function to convert between js_sys::Map (possibly undefined) and Rust HashMap
    pub fn js_map_to_hash_map(map: &js_sys::Map) -> Option<HashMap<String, String>> {
        match map.is_undefined() {
            true => None,
            false => {
                let mut ret = HashMap::<String, String>::new();
                map.for_each(&mut |value, key| {
                    if let Some(key) = key.as_string() {
                        if let Some(value) = value.as_string() {
                            ret.insert(key, value);
                        }
                    }
                });
                Some(ret)
            }
        }
    }

    /// Reads the given package.json file and determines its version.
    #[wasm_bindgen]
    pub fn get_package_version(package_file: &str) -> Result<String, JsValue> {
        ok_or_jserr!(super::get_package_version(package_file))
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }

    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => (utils_misc::utils::wasm::log(&format_args!($($t)*).to_string()))
    }
}
