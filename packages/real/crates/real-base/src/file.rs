pub mod native {
    use crate::error::{RealError, Result};
    use std::fs;

    pub fn read_to_string(file_path: &str) -> Result<String> {
        fs::read_to_string(file_path)
            .map_err(|e| RealError::GeneralError(format!("Failed to read the file '{}' with error: {}", file_path, e)))
    }

    pub fn read_file(file_path: &str) -> Result<Box<[u8]>> {
        match fs::read(file_path) {
            Ok(buf) => Ok(Box::from(buf)),
            Err(e) => Err(RealError::GeneralError(format!(
                "Failed to read the file '{}' with error: {}",
                file_path, e
            ))),
        }
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        fs::write(path, contents).map_err(|e| RealError::GeneralError(format!("{} {}", path, e)))
    }

    pub fn metadata(path: &str) -> Result<()> {
        match fs::metadata(path) {
            Ok(_) => Ok(()), // currently not interested in details
            Err(e) => Err(RealError::GeneralError(e.to_string())),
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::error::{RealError, Result};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen(module = "node:fs")]
    extern "C" {
        #[wasm_bindgen(catch, js_name = "readFileSync")]
        pub fn read_file_js(path: &str) -> std::result::Result<Box<[u8]>, JsValue>;

        #[wasm_bindgen(catch, js_name = "writeFileSync")]
        pub fn write_file_js(path: &str, contents: &[u8]) -> std::result::Result<(), JsValue>;

        #[wasm_bindgen(catch, js_name = "accessSync")]
        pub fn access_js(path: &str) -> std::result::Result<u32, JsValue>;
    }

    #[allow(dead_code)]
    // Copied from Node.js
    enum NodeJsFsConstants {
        /// File exists
        #[allow(non_camel_case_types)]
        F_OK = 0,
        /// File is executable
        #[allow(non_camel_case_types)]
        X_OK = 1,
        /// File is writable
        #[allow(non_camel_case_types)]
        W_OK = 2,
        /// File is readable
        #[allow(non_camel_case_types)]
        R_OK = 4,
    }

    pub fn read_file(path: &str) -> Result<Box<[u8]>> {
        read_file_js(path).map_err(RealError::from)
    }

    pub fn read_to_string(file_path: &str) -> Result<String> {
        let data = read_file(file_path).map_err(|e| RealError::JsError(format!("{:?}", e)))?;
        let text = std::str::from_utf8(&data).map_err(|e| RealError::GeneralError(format!("{:?}", e)))?;
        Ok(text.to_owned())
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        write_file_js(path, contents.as_ref()).map_err(|e| RealError::JsError(format!("{:?}", e)))
    }

    pub fn metadata(path: &str) -> Result<()> {
        match access_js(path) {
            Ok(_) => Ok(()), // currently not interested in details
            Err(e) => Err(RealError::JsError(format!("{:?}", e))),
        }
    }
}
