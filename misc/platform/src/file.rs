pub mod native {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::error::{PlatformError, Result};

    pub fn read_to_string(file_path: &str) -> Result<String> {
        fs::read_to_string(file_path)
            .map_err(|e| PlatformError::GeneralError(format!("Failed to read the file '{file_path}' with error: {e}")))
    }

    pub fn read_file(file_path: &str) -> Result<Box<[u8]>> {
        match fs::read(file_path) {
            Ok(buf) => Ok(Box::from(buf)),
            Err(e) => Err(PlatformError::GeneralError(format!(
                "Failed to read the file '{file_path}' with error: {e}"
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

    pub fn remove_dir_all(path: &std::path::Path) -> Result<()> {
        fs::remove_dir_all(path).map_err(|e| PlatformError::GeneralError(e.to_string()))
    }

    pub fn write<R>(path: &str, contents: R) -> Result<()>
    where
        R: AsRef<[u8]>,
    {
        if let Some(parent_dir_path) = Path::new(path).parent()
            && !parent_dir_path.is_dir() {
                fs::create_dir_all(parent_dir_path)
                    .map_err(|e| PlatformError::GeneralError(format!("Failed to create dir '{path}': {e}")))?
            }
        fs::write(path, contents)
            .map_err(|e| PlatformError::GeneralError(format!("Failed to write to file '{path}': {e}")))
    }

    pub fn metadata(path: &str) -> Result<()> {
        match fs::metadata(path) {
            Ok(_) => Ok(()), // currently not interested in details
            Err(e) => Err(PlatformError::GeneralError(e.to_string())),
        }
    }
}
