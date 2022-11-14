use std::fs;
use std::convert::AsRef;
use std::path::Path;

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    let metadata = fs::metadata(path);

    match metadata {
        Ok(md) => md.is_dir() || md.is_file(),
        Err(_) => false
    }
}