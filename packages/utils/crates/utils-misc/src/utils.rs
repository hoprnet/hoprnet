use std::fmt::Display;
use wasm_bindgen::prelude::*;
use serde::{Deserialize};
use real_base::real;

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String
}

/// Helper function to convert string-convertible types (like errors) to JsValue
fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
    JsValue::from(v.to_string())
}

/// Reads the given package.json file and determines its version.
#[wasm_bindgen]
pub fn get_package_version(package_file: &str) -> Result<String, JsValue> {
    let file_data = real::read_file(package_file)?;

    return serde_json::from_slice::<PackageJsonFile>(&*file_data)
        .map(|v| v.version)
        .map_err(as_jsvalue);
}
