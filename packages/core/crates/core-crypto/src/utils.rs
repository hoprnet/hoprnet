use std::fmt::Display;
use wasm_bindgen::JsValue;

pub fn as_jsvalue<T>(v: T) -> JsValue where T: Display { v.to_string().into() }

pub type JsResult<T> = Result<T, JsValue>;