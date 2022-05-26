use serde::{Deserialize};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn read_file(s: &str) -> Box<[u8]>;
}

#[derive(Deserialize)]
struct PackageJsonFile {
    version: String
}

#[wasm_bindgen]
pub fn get_hoprd_version() -> Result<String, JsValue> {

    let file_contents = read_file("packages/hoprd/package.json");
    if let Ok(v) = serde_json::from_slice::<PackageJsonFile>(&*file_contents) {
        return Ok(v.version);
    }

    Err(JsValue::from("Failed to parse package.json"))
}

#[wasm_bindgen]
pub fn get_version() -> String {
    String::from("1")
}
