use real_base::error::RealError;
use real_base::error::RealError::GeneralError;
use real_base::real;
use serde::Deserialize;

/// Serialization structure for package.json
#[derive(Deserialize)]
struct PackageJsonFile {
    version: String,
}

/// Reads the given package.json file and determines its version.
pub fn get_package_version(package_file: &str) -> Result<String, RealError> {
    let file_data = real::read_file(package_file)?;

    match serde_json::from_slice::<PackageJsonFile>(&*file_data) {
        Ok(package_json) => Ok(package_json.version),
        Err(e) => Err(GeneralError(e.to_string())),
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::ok_or_jserr;
    use paste::paste;
    use wasm_bindgen::prelude::*;

    pub type JsResult<T> = Result<T, JsValue>;

    /// Reads the given package.json file and determines its version.
    #[wasm_bindgen]
    pub fn get_package_version(package_file: &str) -> Result<String, JsValue> {
        ok_or_jserr!(super::get_package_version(package_file))
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
    }

    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => (utils_misc::utils::wasm::log(&format_args!($($t)*).to_string()))
    }

    /// Creates an iterable type in for JavaScript for the given type.
    /// This helps passing vectors in and out of wasm_bindgen bound functions.
    #[macro_export]
    macro_rules! make_jsiterable {
        ($t: ident) => {
            paste! {
                #[wasm_bindgen(getter_with_clone)]
                pub struct [<$t IterableNext>] {
                    pub value: Option<$t>,
                    pub done: bool,
                }

                #[wasm_bindgen]
                pub struct [<$t Iterable>] {
                    backend: std::collections::VecDeque<$t>
                }

                #[wasm_bindgen]
                impl [<$t Iterable>] {
                    pub fn next(&mut self) -> [<$t IterableNext>] {
                        let ret = self.backend.pop_front();
                        [<$t IterableNext>] {
                            done: ret.is_none(),
                            value: ret,
                        }
                    }

                    pub fn count(&self) -> u32 { self.backend.len() as u32 }
                }

                impl From<Vec<$t>> for [<$t Iterable>] {
                    fn from(v: Vec<$t>) -> Self {
                        Self {
                            backend: v.into()
                        }
                    }
                }
            }
        };
    }
}
