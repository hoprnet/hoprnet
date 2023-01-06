use std::fmt::Display;
use wasm_bindgen::JsValue;

pub fn as_jsvalue<T>(v: T) -> JsValue where T: Display {
    JsValue::from(v.to_string())
}

pub type JsResult<T> = Result<T, JsValue>;