use crate::utils::HelperErrors;
use hoprd_keypair::key_pair::HoprKeys;
use log::warn;
use std::{
    fs,collections::HashMap,
    path::{Path, PathBuf},
};

/// Decrypt identity files and returns an vec of PeerIds and Ethereum Addresses
///
/// # Arguments
///
/// * `identity_directory` - Directory to all the identity files
/// * `password` - Password to unlock all the identity files
/// * `identity_prefix` - Prefix of identity files. Only identity files with the provided are decrypted with the password
pub fn read_identities(files: Vec<PathBuf>, password: &String) -> Result<HashMap<String, HoprKeys>, HelperErrors> {
    let mut results = HashMap::with_capacity(files.len());

    for file in files.iter() {
        let file_str = file
            .to_str()
            .ok_or(HelperErrors::IncorrectFilename(file.to_string_lossy().to_string()))?;

        match HoprKeys::read_eth_keystore(file_str, password) {
            Ok((keys, needs_migration)) => {
                if needs_migration {
                    keys.write_eth_keystore(file_str, password, false)?
                }
                let file_key = file.file_name().unwrap();
                results.insert(String::from(file_key.to_str().unwrap()), keys);
            }
            Err(e) => {
                warn!("Could not decrypt keystore file at {}. {}", file_str, e.to_string())
            }
        }
    }

    Ok(results)
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
    maybe_name: &Option<String>,
) -> Result<(String, HoprKeys), HelperErrors> {
    // create dir if not exist
    fs::create_dir_all(dir_name)?;

    let keys = HoprKeys::random();

    // check if `name` is end with `.id`, if not, append it
    let file_path = match maybe_name {
        Some(name) => {
            // check if ending with `.id`
            if name.ends_with(".id") {
                format!("{dir_name}/{name}")
            } else {
                format!("{dir_name}/{name}.id")
            }
        }
        None => format!("{dir_name}/{}.id", { keys.id().to_string() }),
    };

    let path = Path::new(&file_path);
    if path.exists() {
        return Err(HelperErrors::IdentityFileExists(file_path));
    } else {
        keys.write_eth_keystore(&file_path, password, false)?;
    }

    path.file_name()
        .and_then(|p| p.to_str())
        .map(|s| (String::from(s), keys))
        .ok_or(HelperErrors::UnableToCreateIdentity)
}

#[cfg(test)]
mod tests {
    use core_crypto::keypairs::Keypair;
    use std::path::Path;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn create_identities_from_directory_with_id_files() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "password_create";
        match create_identity(path, pwd, &Some(String::from("node1"))) {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "password";
        let (_, created_id) = create_identity(path, pwd, &None).unwrap();

        // created and the read id is identical
        let files = get_files(path, &None);
        let read_id = read_identities(files, &pwd.to_string()).unwrap();
        assert_eq!(read_id.len(), 1);
        assert_eq!(
            read_id.values().nth(0).unwrap().chain_key.public().0.to_address(),
            created_id.chain_key.public().0.to_address()
        );

        // print the read id
        println!("Debug {:#?}", read_id);
        println!("Display {}", read_id.values().nth(0).unwrap());

        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files_but_wrong_password() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "password";
        let wrong_pwd = "wrong_password";
        create_identity(path, pwd, &None).unwrap();
        let files = get_files(path, &None);
        match read_identities(files, &wrong_pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_directory_without_id_files() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let files = get_files(path, &None);
        match read_identities(files, &"".to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_identities_from_tmp_folder() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into())).unwrap();
        let files = get_files(path, &None);
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_prefix() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into())).unwrap();
        let files = get_files(path, &Some("local".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_no_match() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into())).unwrap();
        let files = get_files(path, &Some("npm-".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_wrong_prefix() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let pwd = "local";
        create_identity(path, pwd, &Some("local-alice".into())).unwrap();

        let files = get_files(path, &Some("alice".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_complete_identities_from_tmp_folder() {
        let tmp = tempdir().unwrap();

        let path = tmp.path().to_str().unwrap();
        let name = "alice.id";
        let pwd = "e2e-test";

        let weak_crypto_alice_keystore = r#"{"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"6084fab56497402930d0833fbc17e7ea"},"ciphertext":"50c0cf2537d7bc0ab6dbb7909d21d3da6445e5bd2cb1236de7efbab33302ddf1dd6a0393c986f8c111fe73a22f36af88858d79d23882a5f991713cb798172069d060f28c680afc28743e8842e8e849ebc21209825e23465afcee52a49f9c4f6734061f91a45b4cc8fbd6b4c95cc4c1b487f0007ed88a1b46b5ebdda616013b3f7ba465f97352b9412e69e6690cee0330c0b25bcf5fc3cdf12e4167336997920df9d6b7d816943ab3817481b9","kdf":"scrypt","kdfparams":{"dklen":32,"n":2,"p":1,"r":8,"salt":"46e30c2d74ba04b881e99fb276ae6a970974499f6abe286a00a69ba774ace095"},"mac":"70dccb366e8ddde13ebeef9a6f35bbc1333176cff3d33a72c925ce23753b34f4"},"id":"b5babdf4-da20-4cc1-9484-58ea24f1b3ae","version":3}"#;
        //let alice_peer_id = "16Uiu2HAmUYnGY3USo8iy13SBFW7m5BMQvC4NETu1fGTdoB86piw7";
        let alice_address = "0x838d3c1d2ff5c576d7b270aaaaaa67e619217aac";

        // create dir if not exist.
        fs::create_dir_all(path).unwrap();
        // save the keystore as file
        fs::write(PathBuf::from(path).join(&name), weak_crypto_alice_keystore.as_bytes()).unwrap();

        let files = get_files(path, &None);
        let val = read_identities(files, &pwd.to_string()).unwrap();
        assert_eq!(val.len(), 1);
        assert_eq!(
            val.values()
                .nth(0)
                .unwrap()
                .chain_key
                .public()
                .0
                .to_address()
                .to_string(),
            alice_address
        );

        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    fn remove_json_keystore(path: &str) -> Result<(), HelperErrors> {
        println!("remove_json_keystore {:?}", path);
        match fs::remove_dir_all(path) {
            Ok(_) => Ok(()),
            _ => Err(HelperErrors::UnableToDeleteIdentity),
        }
    }

    fn get_files(identity_directory: &str, identity_prefix: &Option<String>) -> Vec<PathBuf> {
        // early return if failed in reading identity directory
        let directory = fs::read_dir(Path::new(identity_directory)).unwrap();

        // read all the files from the directory that contains
        // 1) "id" in its name
        // 2) the provided idetity_prefix
        let files: Vec<PathBuf> = directory
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
            })
            .collect();
        files
    }
}
