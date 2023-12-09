pub mod time;
pub mod utils;

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn utils_misc_initialize_crate() {
        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}

/// Macro used to convert `Vec<JsString>` to `Vec<&str>`
#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! convert_from_jstrvec {
    ($v:expr,$r:ident) => {
        let _aux: Vec<String> = $v.iter().map(String::from).collect();
        let $r = _aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
    };
}

/// Macro used to convert `Vec<&str>` or `Vec<String>` to `Vec<JString>`
#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! convert_to_jstrvec {
    ($v:expr) => {
        $v.iter().map(|e| js_sys::JsString::from(e.as_ref())).collect()
    };
}

#[macro_export]
macro_rules! clean_mono_repo_path {
    ($v:expr,$r:ident) => {
        let $r = $v.strip_suffix("/").unwrap_or($v);
    };
}

#[macro_export]
macro_rules! ok_or_jserr {
    ($v:expr) => {
        $v.map_err(|e| wasm_bindgen::JsValue::from(e.to_string()))
    };
}
