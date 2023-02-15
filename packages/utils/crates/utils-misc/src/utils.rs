use real_base::error::RealError;
use real_base::error::RealError::GeneralError;
use real_base::real;
use serde::Deserialize;

#[cfg(feature = "wasm")]
pub fn get_time_millis() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(feature = "wasm"))]
pub fn get_time_millis() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
}

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

/// Reads the given package.json file and determines its version.
pub fn get_package_version(package_file: &str) -> Result<String, RealError> {
    let file_data = real::read_file(package_file)?;

    match serde_json::from_slice::<PackageJsonFile>(&*file_data) {
        Ok(package_json) => Ok(package_json.version),
        Err(e) => Err(GeneralError(e.to_string())),
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::ok_or_jserr;
    use wasm_bindgen::prelude::*;

    pub type JsResult<T> = Result<T, JsValue>;

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
