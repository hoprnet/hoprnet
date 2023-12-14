pub mod native {
    use crate::error::{PlatformError, Result};
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    pub fn read_to_string(file_path: &str) -> Result<String> {
        fs::read_to_string(file_path)
            .map_err(|e| PlatformError::GeneralError(format!("Failed to read the file '{}' with error: {}", file_path, e)))
    }

    pub fn read_file(file_path: &str) -> Result<Box<[u8]>> {
        match fs::read(file_path) {
            Ok(buf) => Ok(Box::from(buf)),
            Err(e) => Err(PlatformError::GeneralError(format!(
                "Failed to read the file '{}' with error: {}",
                file_path, e
            ))),
        }
    }

    pub fn join(components: &[&str]) -> Result<String> {
        let mut path = PathBuf::new();

        for component in components.iter() {
            path.push(component);
        }

        match path.to_str().map(|p| p.to_owned()) {
            Some(p) => Ok(p),
            None => Err(PlatformError::GeneralError("Failed to stringify path".into())),
        }
    }

    pub fn remove_dir_all(path: &str) -> Result<()> {
        fs::remove_dir_all(Path::new(path)).map_err(|e| PlatformError::GeneralError(e.to_string()))?;

        Ok(())
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        fs::write(path, contents).map_err(|e| PlatformError::GeneralError(format!("{} {}", path, e)))
    }

    pub fn metadata(path: &str) -> Result<()> {
        match fs::metadata(path) {
            Ok(_) => Ok(()), // currently not interested in details
            Err(e) => Err(PlatformError::GeneralError(e.to_string())),
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::error::{PlatformError, Result};
    use bitflags::bitflags;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen(module = "node:fs")]
    extern "C" {
        #[wasm_bindgen(catch, js_name = "readFileSync")]
        pub fn read_file_js(path: &str) -> std::result::Result<Box<[u8]>, JsValue>;

        #[wasm_bindgen(catch, js_name = "writeFileSync")]
        pub fn write_file_js(path: &str, contents: &[u8]) -> std::result::Result<(), JsValue>;

        #[wasm_bindgen(catch, js_name = "accessSync")]
        pub fn access_js(path: &str, mode: u32) -> std::result::Result<JsValue, JsValue>;
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(catch, js_name = "removePathRecursively")]
        pub fn remove_dir(path: &str) -> std::result::Result<(), JsValue>;
    }

    // Copied from Node.js
    bitflags! {
        struct NodeJsFsConstants: u32 {
            /// File exists
            #[allow(non_camel_case_types)]
            const F_OK = 0;
            /// File is executable
            #[allow(non_camel_case_types)]
            const X_OK = 1;
            /// File is writable
            #[allow(non_camel_case_types)]
            const W_OK = 2;
            /// File is readable
            #[allow(non_camel_case_types)]
            const R_OK = 4;
        }
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

    pub fn join(components: &[&str]) -> Result<String> {
        // NOTE: expecting a Unix system
        Ok(components.join("/"))
    }

    pub fn remove_dir_all(path: &str) -> Result<()> {
        remove_dir(path).map_err(|e| {
            RealError::GeneralError(
                e.as_string()
                    .unwrap_or(format!("Unknown error on removing the path: {}", path)),
            )
        })
    }

    pub fn metadata(path: &str) -> Result<()> {
        // pass in same mode as default (read + write)
        let mode = (NodeJsFsConstants::W_OK | NodeJsFsConstants::R_OK).bits();
        match access_js(path, mode) {
            Ok(_) => Ok(()),
            Err(e) => Err(RealError::JsError(format!("{:?}", e))),
        }
    }
}
