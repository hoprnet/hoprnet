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

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    macro_rules! ok_or_jserr {
        ($v:expr) => {
            $v.map_err(|e| JsValue::from(e.to_string()))
        };
    }

    /// Reads the given package.json file and determines its version.
    #[wasm_bindgen]
    pub fn get_package_version(package_file: &str) -> Result<String, JsValue> {
        ok_or_jserr!(super::get_package_version(package_file))
    }
}
