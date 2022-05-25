use serde::{Deserialize};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn read_file_to_string(s: &str) -> Option<String>;
}

#[derive(Deserialize)]
struct PackageJsonFile {
    version: String
}

#[wasm_bindgen]
pub fn get_hoprd_version() -> Result<String, JsValue> {

    if let Some(file_contents) = read_file_to_string("packages/hoprd/package.json") {
        if let Ok(v) = serde_json::from_str::<PackageJsonFile>(file_contents.as_str()) {
            return Ok(v.version);
        }
    }

    Err(JsValue::from("Failed to parse package.json"))
}

