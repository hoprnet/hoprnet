pub mod native {
    use crate::error::{RealError, Result};
    use std::fs;

    pub fn read_to_string(file_path: &str) -> Result<String> {
        fs::read_to_string(file_path).map_err(|e| {
            RealError::GeneralError(format!(
                "Failed to read the file '{}' with error: {}",
                file_path,
                e.to_string()
            ))
        })
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        fs::write(path, contents).map_err(|e| RealError::GeneralError(format!("{} {}", path, e.to_string())))
    }

    pub fn metadata(path: &str) -> Result<()> {
        fs::metadata(path).map_err(|e| RealError::GeneralError(e.to_string()))?;
        Ok(())
    }
}

pub mod wasm {
    use crate::error::{RealError, Result};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "node:fs")]
    extern "C" {
        #[wasm_bindgen(catch, js_name = "readFileSync")]
        pub fn read_file(path: &str) -> std::result::Result<Box<[u8]>, JsValue>;

        #[wasm_bindgen(catch, js_name = "writeFileSync")]
        pub fn write_file(path: &str, contents: &[u8]) -> std::result::Result<(), JsValue>;

        #[wasm_bindgen(catch)]
        pub fn access(path: &str) -> std::result::Result<u32, JsValue>;
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

    pub fn read_to_string(file_path: &str) -> Result<String> {
        let data = read_file(file_path).map_err(|e| RealError::JsError(format!("{:?}", e)))?;
        let text = std::str::from_utf8(&data).map_err(|e| RealError::GeneralError(format!("{:?}", e)))?;
        Ok(text.to_owned())
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        write_file(path, contents.as_ref()).map_err(|e| RealError::JsError(format!("{:?}", e)))
    }

    pub fn metadata(path: &str) -> Result<()> {
        access(path).map_err(|e| RealError::JsError(format!("{:?}", e)))?;
        Ok(())
    }
}
