use core_crypto::checksum::to_checksum;
use core_crypto::types::PublicKey;
use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
use ethers::core::{
    k256::{ecdsa::SigningKey, PublicKey as K256PublicKey},
    rand::thread_rng,
    types::H256,
};
use ethers::prelude::k256::ecdsa::VerifyingKey;
use ethers::signers::Signer;
use ethers::signers::Wallet;
use ethers::types::Address;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct NodeIdentity {
    pub peer_id: String,
    pub ethereum_address: String,
}

impl NodeIdentity {
    // pub fn new(verifying_key: VerifyingKey) -> Self {
    pub fn new(signing_key: Wallet<SigningKey>) -> Self {
        let verifying_key = signing_key.signer().verifying_key();
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
        write!(f, "PeerId: {}, Address: {}", self.peer_id, self.ethereum_address)
    }
}

/// Decrypt identity files and returns an vec of PeerIds and Ethereum Addresses
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
) -> Result<Vec<NodeIdentity>, std::io::Error> {
    // early return if failed in reading identity directory
    let directory = fs::read_dir(Path::new(identity_directory))?;

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

    // get the verifying key from each file
    let signing_keys: Vec<Wallet<SigningKey>> = files
        .into_iter()
        .filter_map(|r| Wallet::<SigningKey>::decrypt_keystore(r, password).ok())
        .collect();

    // convert verifying_keys to NodeIdentity
    let results: Vec<NodeIdentity> = signing_keys.into_iter().map(|r| NodeIdentity::new(r)).collect();

    Ok(results)
}

/// Create one identity file and return the ethereum address
///
/// # Arguments
///
/// * `dir_name` - Directory to the storage of an identity file
/// * `password` - Password to encrypt the identity file
/// * `name` - Prefix of identity files.
pub fn create_identity(dir_name: &str, password: &str, name: &Option<String>) -> Result<Address, std::io::Error> {
    // create dir if not exist
    fs::create_dir_all(dir_name)?;

    // create identity with the given password
    let (wallet, uuid) = Wallet::new_keystore(Path::new(dir_name), &mut thread_rng(), password, None).unwrap();

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

    use std::{fs::File, io::Write};

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
        create_identity(path, pwd, &None).unwrap();
        match read_identities(path, &pwd.to_string(), &None) {
            Ok(val) => assert_eq!(val.len(), 1),
            _ => assert!(false),
        }
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
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
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
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
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
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
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
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
        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
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
        fs::write(PathBuf::from(path).join(&name), weak_crypto_alice_keystore.as_bytes());

        let val = read_identities(path, &pwd.to_string(), &None).unwrap();
        assert_eq!(val.len(), 1);
        assert_eq!(val[0].peer_id, alice_peer_id);
        assert_eq!(val[0].ethereum_address, alice_address);

        remove_json_keystore(path).map_err(|err| println!("{:?}", err)).ok();
    }

    fn remove_json_keystore(path: &str) -> Result<(), std::io::Error> {
        println!("remove_json_keystore {:?}", path);
        fs::remove_dir_all(path)
    }
}
