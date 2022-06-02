mod utils;

use wasm_bindgen::prelude::*;

use hopr_real::real;

use serde::{Deserialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

