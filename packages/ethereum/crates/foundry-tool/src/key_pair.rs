use std::path::{Path, PathBuf};
use std::fs;
use std::io;

pub fn read_identities (identity_directory: &str, identity_prefix: Option<String>) -> Result<Vec<PathBuf>, io::Error> {
    
    // read all the files from the directory
  // Check if local identity files should be used. Push all the read identities.
  let file_names = fs::read_dir(Path::new(identity_directory))?
    .into_iter()
    .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
    .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
    .filter(|r| r.is_file() && r.extension().unwrap() == "id") // Filter out folders
    .collect();
    
  println!("file_names {:?}", file_names);

  Ok(file_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_identities_from_directory_with_id_files() {
        match read_identities("/tmp", None) {
            Ok(val) => assert_eq!(val.len(), 2),
            _ => assert!(false)
        }
    }
    #[test]
    fn read_identities_from_directory_without_id_files() {
        match read_identities("/var", None) {
            Ok(val) => assert_eq!(val.len(), 0),
            // Ok(10) => assert!(true),
            _ => assert!(false)
        }
        // assert_eq!(read_identities("/tmp", None).is_ok_and(|&x| x ==10), 10);
    }
}