use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@hoprnet/hopr-wasm")]
extern "C" {

    #[wasm_bindgen(js_name = DataOrError , typescript_type = "DataOrError")]
    pub type DataOrError;

    #[wasm_bindgen(method, getter)]
    fn data(this: &DataOrError) -> Box<[u8]>;

    #[wasm_bindgen(method, getter)]
    fn error(this: &DataOrError) -> JsValue;

    /// Reads the given file and returns it as array of bytes.
    pub fn read_file(file: &str) -> DataOrError;
}

impl From<DataOrError> for Result<Vec<u8>, JsValue> {
    fn from(d: DataOrError) -> Self {
        let e = d.error();
        return if e.is_undefined() {
            Ok(Vec::from(d.data()))
        } else {
            Err(e)
        }
    }
}