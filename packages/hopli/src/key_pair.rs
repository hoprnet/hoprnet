use crate::utils::HelperErrors;
use core_crypto::checksum::to_checksum;
use core_crypto::types::PublicKey;
use elliptic_curve::rand_core::{CryptoRng, RngCore};
use eth_keystore;
use ethers::core::rand::thread_rng;
use generic_array::GenericArray;
use k256::ecdsa::{SigningKey, VerifyingKey};
use serde::Serialize;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub struct NodeIdentity {
    pub peer_id: String,
    pub ethereum_address: String,
}

impl NodeIdentity {
    pub fn new(verifying_key: VerifyingKey) -> Self {
        let public_key = PublicKey::deserialize(&verifying_key.to_encoded_point(false).to_bytes()).unwrap();

        // derive PeerId
        let id = public_key._to_peerid_str();

        // derive ethereum address
        let address = to_checksum(public_key.to_address());

        Self {
            peer_id: id,
            ethereum_address: address,
        }
    }
}

impl fmt::Display for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "peer_id: {}, ethereum_address: {}",
            self.peer_id, self.ethereum_address
        )
    }
}

/// Decrypt identity files and returns an vec of PeerIds and Ethereum Addresses
///
/// # Arguments
///
/// * `identity_directory` - Directory to all the identity files
/// * `password` - Password to unlock all the identity files
/// * `identity_prefix` - Prefix of identity files. Only identity files with the provided are decrypted with the password
pub fn read_identities(files: Vec<PathBuf>, password: &String) -> Result<Vec<NodeIdentity>, HelperErrors> {
    // get the verifying key from each file
    let signing_keys: Vec<VerifyingKey> = files
        .into_iter()
        .filter_map(|r| decrypt_keystore(r, password).ok())
        .collect();

    // convert verifying_keys to NodeIdentity
    let results: Vec<NodeIdentity> = signing_keys.into_iter().map(|r| NodeIdentity::new(r)).collect();

    Ok(results)
}

fn decrypt_keystore<P, S>(keypath: P, password: S) -> Result<VerifyingKey, HelperErrors>
where
    P: AsRef<Path>,
    S: AsRef<[u8]>,
{
    let secret = eth_keystore::decrypt_key(keypath, password)?;
    let signer = SigningKey::from_bytes(GenericArray::from_slice(&secret))?;
    Ok(*signer.verifying_key())
}

fn new_keystore<P, R, S>(dir: P, rng: &mut R, password: S) -> Result<(VerifyingKey, String), HelperErrors>
where
    P: AsRef<Path>,
    R: CryptoRng + RngCore,
    S: AsRef<[u8]>,
{
    let (secret, uuid) = eth_keystore::new(dir, rng, password, None)?;
    let signer = SigningKey::from_bytes(GenericArray::from_slice(&secret))?;
    Ok((*signer.verifying_key(), uuid))
}

/// Create one identity file and return the ethereum address
///
/// # Arguments
///
/// * `dir_name` - Directory to the storage of an identity file
/// * `password` - Password to encrypt the identity file
/// * `name` - Prefix of identity files.
pub fn create_identity(dir_name: &str, password: &str, name: &Option<String>) -> Result<NodeIdentity, HelperErrors> {
    // create dir if not exist
    fs::create_dir_all(dir_name)?;

    // create identity with the given password
    let (verifying_key, uuid) = new_keystore(Path::new(dir_name), &mut thread_rng(), password)?;

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

    Ok(NodeIdentity::new(verifying_key))
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
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files() {
        let path = "./tmp_1";
        let pwd = "password";
        let created_id = create_identity(path, pwd, &None).unwrap();

        // created and the read id is identical
        let files = get_files(path, &None);
        let read_id = read_identities(files, &pwd.to_string()).unwrap();
        assert_eq!(read_id.len(), 1);
        assert_eq!(read_id[0].ethereum_address, created_id.ethereum_address);
        assert_eq!(read_id[0].peer_id, created_id.peer_id);

        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_directory_with_id_files_but_wrong_password() {
        let path = "./tmp_2";
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
        let path = "./";
        let files = get_files(path, &None);
        match read_identities(files, &"".to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
    }

    #[test]
    fn read_identities_from_tmp_folder() {
        let path = "./tmp_4";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        let files = get_files(path, &None);
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_prefix() {
        let path = "./tmp_5";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        let files = get_files(path, &Some("local".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_no_match() {
        let path = "./tmp_6";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();
        let files = get_files(path, &Some("npm-".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_identities_from_tmp_folder_with_wrong_prefix() {
        let path = "./tmp_7";
        let pwd = "local";
        create_identity(path, pwd, &Some(String::from("local-alice"))).unwrap();

        let files = get_files(path, &Some("alice".to_string()));
        match read_identities(files, &pwd.to_string()) {
            Ok(val) => assert_eq!(val.len(), 0),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    #[test]
    fn read_complete_identities_from_tmp_folder() {
        let path = "./tmp_8";
        let name = "alice.id";
        let pwd = "local";

        let weak_crypto_alice_keystore = r#"{"id":"8e5fe142-6ef9-4fbb-aae8-5de32b680e31","version":3,"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"04141354edb9dfb0c65e6905a3a0b9dd"},"ciphertext":"74f12f72cf2d3d73ff09f783cb9b57995b3808f7d3f71aa1fa1968696aedfbdd","kdf":"scrypt","kdfparams":{"salt":"f5e3f04eaa0c9efffcb5168c6735d7e1fe4d96f48a636c4f00107e7c34722f45","n":1,"dklen":32,"p":1,"r":8},"mac":"d0daf0e5d14a2841f0f7221014d805addfb7609d85329d4c6424a098e50b6fbe"}}"#;
        let alice_peer_id = "16Uiu2HAm8WFpakjrdWauUKq2hb5bejivnbtFAumVv9KHKN5AvXXK";
        let alice_address = "0x826A1bF3d51Fa7F402A1E01d1B2C8A8bAC28E666";

        // create dir if not exist.
        fs::create_dir_all(path).unwrap();
        // save the keystore as file
        fs::write(PathBuf::from(path).join(&name), weak_crypto_alice_keystore.as_bytes()).unwrap();

        let files = get_files(path, &None);
        let val = read_identities(files, &pwd.to_string()).unwrap();
        assert_eq!(val.len(), 1);
        assert_eq!(val[0].peer_id, alice_peer_id);
        assert_eq!(val[0].ethereum_address, alice_address);

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
