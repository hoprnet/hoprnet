pub mod native {
    use std::fs;

    pub fn read_to_string(file_path: &str) -> Result<String, String> {
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read the file '{}' with error: {}", file_path, e.to_string()))
    }
}

pub mod wasm {
    pub fn read_to_string(file_path: &str) -> Result<String, String> {
        let data = crate::real::read_file(file_path).map_err(|e| e.to_string())?;
        let text = std::str::from_utf8(&data).map_err(|e| e.to_string())?;
        Ok(text.to_owned())
    }
}
