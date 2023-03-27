use semver::Version;
use real_base::error::RealError;
use real_base::error::RealError::GeneralError;
use real_base::real;
use serde::Deserialize;

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

/// Represents a version number simplified to Major, Minor and Patch.
pub type MajMinPatch = [u8; 3];

/// Parses the Semver package version from the package.json file and converts it
/// to a simplified Major, Minor and Patch.
pub fn parse_package_version(package_file: &str) -> Result<MajMinPatch, RealError> {
    get_package_version(package_file)
        .and_then(|v| Version::parse(v.as_str())
            .map_err(|e| GeneralError(e.to_string()))
            .map(|v| [v.major as u8, v.minor as u8, v.patch as u8]))
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
