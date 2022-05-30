use wasm_bindgen::prelude::*;

use crate::real;

use serde::{Deserialize};

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String
}

/// Reads the package.json file of hoprd and determines it's version.
#[wasm_bindgen]
pub fn get_hoprd_version() -> Result<String, JsValue> {

    let file_data = real::read_file("packages/hoprd/package.json");

    return serde_json::from_slice::<PackageJsonFile>(Result::from(file_data)?.as_slice())
        .map(|v| v.version)
        .map_err(|e| JsValue::from(e.to_string()));
}
