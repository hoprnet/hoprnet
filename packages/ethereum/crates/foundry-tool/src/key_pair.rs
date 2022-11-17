use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use ethers::signers::Signer;
use ethers::signers::Wallet;
use ethers::core::rand::thread_rng;
use ethers::types::Address;

pub fn read_identities (identity_directory: &str, password: &String, identity_prefix: Option<String>) -> Result<Vec<Address>, io::Error> {
  let addresses = fs::read_dir(Path::new(identity_directory))?
    .into_iter() // read all the files from the directory
    .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
    .map(|r| r.unwrap().path()) // Read all the files from the given directory
    .filter(|r| r.is_file() && r.extension().unwrap() == "id") // Filter out folders
    .map(|r| Wallet::decrypt_keystore(r, password).unwrap().address()) // read keystore and return address
    .collect();
    
  println!("Addresses from identities {:?}", addresses);
  Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_identities_from_directory_with_id_files() {
        let path = "./tmp";
        let pwd = "password";
        create_json_keystore(path, pwd).unwrap();
        match read_identities(path, &pwd.to_string(), None) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false)
        }
        remove_json_keystore(path);
    }
    #[test]
    fn read_identities_from_directory_without_id_files() {
        match read_identities("./", &"".to_string(), None) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false)
        }
    }

    fn create_json_keystore(dir_name: &str, pwd: &str) -> Result<(), io::Error> {
      fs::create_dir_all(dir_name)?;
      let (key, uuid) = Wallet::new_keystore(Path::new(&dir_name), &mut thread_rng(), pwd, None).unwrap();
      let old_file_path = vec![dir_name, "/", &*uuid].concat();
      let new_file_path = vec![dir_name, "/", &*uuid, ".id"].concat();
      fs::rename(&old_file_path, &new_file_path); // Rename keystore from uuid to uuid.id
      Ok(())
    }
    
    fn remove_json_keystore(path: &str) -> Result<(), io::Error> {
        println!("remove_json_keystore {:?}", path);
        fs::remove_dir_all(path)
    }
}