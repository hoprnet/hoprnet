#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::{Object, Reflect, Uint8Array};
    use wasm_bindgen::prelude::*;

    /// Turns a JsValue stream into a Box<[u8]> stream
    ///
    /// ```
    /// use futures::stream::{Stream,StreamExt};
    /// use js_sys::Uint8Array;
    /// use utils_misc::async_iterable::wasm::to_box_u8_stream;
    /// use wasm_bindgen::prelude::*;
    ///
    /// pub async fn take_stream(stream: impl Stream<Item = Result<JsValue,JsValue>>) -> Vec<Result<Box<[u8]>,String>> {
    ///   stream.map(to_box_u8_stream).collect::<Vec<Result<Box<[u8]>,String>>>().await
    /// }
    /// ```
    pub fn to_box_u8_stream(item: Result<JsValue, JsValue>) -> Result<Box<[u8]>, String> {
        match item {
            Ok(x) => Ok(Uint8Array::from(x).to_vec().into_boxed_slice()),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    /// Transforms the output of a Rust stream into Javascript iterator protocol
    ///
    /// ```
    /// use futures::stream::{Stream,StreamExt};
    /// use utils_misc::async_iterable::wasm::to_jsvalue_stream;
    /// use wasm_bindgen::prelude::*;
    ///
    /// pub async fn take_stream(mut stream: impl Stream<Item = Result<Box<[u8]>,String>> + Unpin) -> Result<JsValue, JsValue> {
    ///    to_jsvalue_stream(stream.next().await)
    /// }
    /// ```
    pub fn to_jsvalue_stream(item: Option<Result<Box<[u8]>, String>>) -> Result<JsValue, JsValue> {
        let obj = Object::new();

        match item {
            Some(Ok(m)) => {
                Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
                Reflect::set(&obj, &"value".into(), &Uint8Array::from(m.as_ref())).unwrap();
                Ok(obj.into())
            }
            Some(Err(e)) => Err(JsValue::from(e)),
            None => {
                Reflect::set(&obj, &"done".into(), &JsValue::TRUE).unwrap();
                Reflect::set(&obj, &"value".into(), &JsValue::undefined()).unwrap();
                Ok(obj.into())
            }
        }
    }

    /// Transforms input into iterator protocol
    ///
    /// ```no_run
    /// # use utils_misc::async_iterable::wasm::to_jsvalue_iterator;
    ///
    /// let first_chunk: Box<[u8]> = Box::new([0u8,1u8]);
    ///
    /// to_jsvalue_iterator(Some(first_chunk));
    ///
    /// // end stream
    /// to_jsvalue_iterator(None);
    /// ```
    pub fn to_jsvalue_iterator(item: Option<Box<[u8]>>) -> JsValue {
        let obj = Object::new();

        match item {
            Some(m) => {
                Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
                Reflect::set(&obj, &"value".into(), &Uint8Array::from(m.as_ref())).unwrap();
                obj.into()
            }
            None => {
                Reflect::set(&obj, &"done".into(), &JsValue::TRUE).unwrap();
                Reflect::set(&obj, &"value".into(), &JsValue::undefined()).unwrap();
                obj.into()
            }
        }
    }
}
