#[cfg(feature = "wasm")]
pub mod wasm {
    /// Macro creating a wrapped chain request value
    ///
    /// The value contains the value itself, along with a receipt string,
    /// all convertible and definable in a wasm-bindgen context
    #[macro_export]
    macro_rules! to_chain_value {
        ($obj:ident,$x:ty) => {
            #[wasm_bindgen]
            #[derive(Debug, Clone)]
            pub struct $obj {
                value: $x,
                receipt: String,
            }

            impl $obj {
                pub fn new(value: $x, receipt: String) -> Self {
                    Self { value, receipt }
                }
            }

            #[wasm_bindgen]
            impl $obj {
                #[wasm_bindgen]
                pub fn value(&self) -> $x {
                    self.value.clone()
                }

                #[wasm_bindgen]
                pub fn receipt(&self) -> js_sys::JsString {
                    js_sys::JsString::from(self.receipt.as_str())
                }
            }
        };
    }
}
