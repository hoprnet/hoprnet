use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@hoprnet/hopr-real")]
extern "C" {

    /*
    DataOrError type is a workaround for the temporary impossibility of
    having Result<Box<[u8]>>,JsValue> as a return value on a function imported
    from JS with the "catch" attribute to handle exceptions.

    Right now, the exceptions must be properly and fully handled in JS.
     */

    #[wasm_bindgen(js_name = DataOrError , typescript_type = "DataOrError")]
    pub type DataOrError;

    #[wasm_bindgen(method, getter)]
    fn data(this: &DataOrError) -> Box<[u8]>;

    #[wasm_bindgen(method, getter)]
    fn error(this: &DataOrError) -> JsValue;

    #[wasm_bindgen(method)]
    fn hasError(this: &DataOrError) -> bool;

    // Reads the given file and returns it as array of bytes.
    pub fn read_file(file: &str) -> DataOrError;
}

impl From<DataOrError> for Result<Vec<u8>, JsValue> {
    fn from(d: DataOrError) -> Self {
        if !d.hasError() {
            Ok(Vec::from(d.data()))
        }
        else {
            Err(d.error())
        }
    }
}