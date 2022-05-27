use wasm_bindgen::prelude::*;

use serde::{Deserialize};

/// Imports from REAL
#[wasm_bindgen(module = "@hoprnet/hopr-wasm")]
extern "C" {
    /// Reads the given file and returns it as array of bytes.
    fn read_file(s: &str) -> Box<[u8]>;
}

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String
}

/// Reads the package.json file of hoprd and determines it's version.
#[wasm_bindgen]
pub fn get_hoprd_version() -> Result<String, JsValue> {

    let file_contents = read_file("packages/hoprd/package.json");
    if let Ok(v) = serde_json::from_slice::<PackageJsonFile>(&*file_contents) {
        return Ok(v.version);
    }

    Err(JsValue::from("Failed to parse package.json"))
}
