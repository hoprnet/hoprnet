use ethers::core::{k256::ecdsa::SigningKey, rand::thread_rng};
use ethers::signers::Signer;
use ethers::signers::Wallet;
use ethers::types::Address;
use std::fs;
use std::path::Path;

/// Decrypt identity files and returns an vec of ethereum addresses
///
/// # Arguments
///
/// * `identity_directory` - Directory to all the identity files
/// * `password` - Password to unlock all the identity files
/// * `identity_prefix` - Prefix of identity files. Only identity files with the provided are decrypted with the password
pub fn read_identities(
    identity_directory: &str,
    password: &String,
    identity_prefix: &Option<String>,
) -> Result<Vec<Address>, std::io::Error> {
    match fs::read_dir(Path::new(identity_directory)) {
        Ok(directory) => {
            let addresses: Vec<Address> = directory
                .into_iter() // read all the files from the directory
                .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
                .map(|r| r.unwrap().path()) // Read all the files from the given directory
                .filter(|r| r.is_file()) // Filter out folders
                .filter(|r| r.to_str().unwrap().contains("id")) // file name should contain "id"
                .filter(|r| match &identity_prefix {
                    Some(identity_prefix) => r
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with(identity_prefix.as_str()),
                    _ => true,
                }) // TODO: Now it is a loose check on contain but not strict on the prefix
                .filter_map(|r| Wallet::<SigningKey>::decrypt_keystore(r, password).ok()) // read keystore and return non-error results
                .map(|r| r.address()) // read keystore and return address
                .collect();

            Ok(addresses)
        }
        Err(e) => Err(e),
    }
}

/// Create one identity file and return the ethereum address
///
/// # Arguments
///
/// * `dir_name` - Directory to the storage of an identity file
/// * `password` - Password to encrypt the identity file
/// * `name` - Prefix of identity files.
pub fn create_identity(
    dir_name: &str,
    password: &str,
    name: &Option<String>,
) -> Result<Address, std::io::Error> {
    // create dir if not exist
    fs::create_dir_all(dir_name)?;

    // create identity with the given password
    let (wallet, uuid) =
        Wallet::new_keystore(Path::new(dir_name), &mut thread_rng(), password, None).unwrap();

    // Rename keystore from uuid to uuid.id (or `name.id`, if provided)
    let old_file_path = vec![dir_name, "/", &*uuid].concat();

    // check if `name` is end with `.id`, if not, append it
    let new_file_path = match name {
        Some(provided_name) => {
            // check if ending with `.id`
            if provided_name.ends_with(".id") {
                vec![dir_name, "/", provided_name].concat()
            } else {
                vec![dir_name, "/", provided_name, ".id"].concat()
            }
        }
        None => vec![dir_name, "/", &*uuid, ".id"].concat(),
    };

    fs::rename(&old_file_path, &new_file_path)
        .map_err(|err| println!("{:?}", err))
        .ok();

    Ok(wallet.address())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_identities_from_directory_with_id_files() {
        let path = "./tmp_create";
        let pwd = "password_create";
        match create_identity(path, pwd, &Some(String::from("node1"))) {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files() {
        let path = "./tmp_1";
        let pwd = "password";
        create_identity(path, pwd, &None).unwrap();
        match read_identities(path, &pwd.to_string(), &None) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files_but_wrong_password() {
        let path = "./tmp_2";
        let pwd = "password";
        let wrong_pwd = "wrong_password";
        create_identity(path, pwd, &None).unwrap();
        match read_identities(path, &wrong_pwd.to_string(), &None) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_directory_without_id_files() {
        let path = "./";
        match read_identities(path, &"".to_string(), &None) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_identities_from_tmp_folder() {
        let path = "./tmp_4";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        match read_identities(path, &pwd.to_string(), &None) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_prefix() {
        let path = "./tmp_5";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        match read_identities(path, &pwd.to_string(), &Some("local".to_string())) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_no_match() {
        let path = "./tmp_6";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        match read_identities(path, &pwd.to_string(), &Some("npm-".to_string())) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_wrong_prefix() {
        let path = "./tmp_7";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        match read_identities(path, &pwd.to_string(), &Some("alice".to_string())) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    fn remove_json_keystore(path: &str) -> Result<(), std::io::Error> {
        println!("remove_json_keystore {:?}", path);
        fs::remove_dir_all(path)
    }
}
