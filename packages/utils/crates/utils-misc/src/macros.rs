
#[cfg(feature = "wasm")]
pub mod wasm {
    /// Macro used to convert Vec<JString> to Vec<&str>
    #[macro_export]
    macro_rules! convert_from_jstrvec {
        ($v:expr,$r:ident) => {
            let _aux: Vec<String> = $v.iter().map(String::from).collect();
            let $r = _aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        };
    }

    /// Macro used to convert Vec<&str> or Vec<String> to Vec<JString>
    #[macro_export]
    macro_rules! convert_to_jstrvec {
        ($v:expr) => {
            $v.iter().map(|e| JsString::from(e.as_ref())).collect()
        };
    }
}