use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::rand::thread_rng;
use ethers::signers::Signer;
use ethers::signers::Wallet;
use ethers::types::Address;
use std::fs;
use std::path::Path;

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
                .filter(|r| match r.extension() {
                    Some(ext) => &ext.to_os_string().into_string().unwrap() == "id",
                    None => false,
                }) // Filter out wrong extension
                // .map(|r| r.into_os_string().into_string().unwrap())
                .filter(|r| match &identity_prefix {
                    Some(identity_prefix) => r
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .contains(identity_prefix.as_str()),
                    _ => true,
                }) // TODO: Now it is a loose check on contain but not strict on the prefix
                .filter_map(|r| Wallet::<SigningKey>::decrypt_keystore(r, password).ok()) // read keystore and return non-error results
                .map(|r| r.address()) // read keystore and return address
                .collect();

            println!("Addresses from identities {:?}", addresses);
            Ok(addresses)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_identities_from_directory_with_id_files() {
        let path = "./tmp_1";
        let pwd = "password";
        create_json_keystore(path, pwd, false).unwrap();
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
        create_json_keystore(path, pwd, false).unwrap();
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
        create_json_keystore(path, pwd, true).unwrap();
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
        create_json_keystore(path, pwd, true).unwrap();
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
        create_json_keystore(path, pwd, true).unwrap();
        match read_identities(path, &pwd.to_string(), &Some("npm-".to_string())) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path)
            .map_err(|err| println!("{:?}", err))
            .ok();
    }

    fn create_json_keystore(
        dir_name: &str,
        pwd: &str,
        local_like: bool,
    ) -> Result<(), std::io::Error> {
        fs::create_dir_all(dir_name)?;
        let (_key, uuid) =
            Wallet::new_keystore(Path::new(dir_name), &mut thread_rng(), pwd, None).unwrap();
        let old_file_path = vec![dir_name, "/", &*uuid].concat();
        let new_file_path = if local_like {
            vec![dir_name, "/local-alice.id"].concat()
        } else {
            vec![dir_name, "/", &*uuid, ".id"].concat()
        };
        fs::rename(&old_file_path, &new_file_path)
            .map_err(|err| println!("{:?}", err))
            .ok(); // Rename keystore from uuid to uuid.id
        Ok(())
    }

    fn remove_json_keystore(path: &str) -> Result<(), std::io::Error> {
        println!("remove_json_keystore {:?}", path);
        fs::remove_dir_all(path)
    }
}
