use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@hoprnet/hopr-real")]
extern "C" {

    // Reads the given file and returns it as array of bytes.
    #[wasm_bindgen(catch)]
    pub fn read_file(file: &str) -> Result<Box<[u8]>, JsValue>;

    #[wasm_bindgen(catch)]
    pub fn write_file(file: &str, data: &[u8]) -> Result<(), JsValue>;
}
