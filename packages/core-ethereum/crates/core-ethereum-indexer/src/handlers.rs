use crate::errors::{CoreEthereumIndexerError, Result};
use bindings::hopr_announcements::HoprAnnouncementsEvents;
use core_crypto::types::OffchainSignature;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::account::{AccountEntry, AccountSignature, AccountType};
use ethers::{contract::EthLogDecode, core::abi::RawLog};
use utils_types::primitives::Snapshot;

pub async fn on_announce<T>(db: &mut T, log: &RawLog, block_number: u32, snapshot: &Snapshot) -> Result<AccountEntry>
where
    T: HoprCoreEthereumDbActions,
{
    Ok(match HoprAnnouncementsEvents::decode_log(log)? {
        HoprAnnouncementsEvents::AddressAnnouncementFilter(address_announcement) => {
            let maybe_account = db.get_account(&address_announcement.node.try_into()?).await?;

            if let Some(mut account) = maybe_account {
                let new_entry_type = AccountType::Announced {
                    multiaddr: address_announcement.base_multiaddr.try_into()?,
                    updated_block: block_number,
                };

                account.update(new_entry_type);
                db.update_account_and_snapshot(&account, snapshot).await?;

                account
            } else {
                return Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding);
            }
        }
        HoprAnnouncementsEvents::KeyBindingFilter(key_binding) => {
            if db.get_account(&key_binding.chain_key.try_into()?).await?.is_some() {
                return Err(CoreEthereumIndexerError::UnsupportedKeyRebinding);
            }

            let updated_account = AccountEntry::new(
                key_binding.ed_25519_pub_key.try_into()?,
                key_binding.chain_key.try_into()?,
                AccountType::NotAnnounced,
            );

            let sig = AccountSignature {
                signature: OffchainSignature::try_from((key_binding.ed_25519_sig_0, key_binding.ed_25519_sig_1))?,
                pub_key: key_binding.ed_25519_pub_key.try_into()?,
                chain_key: key_binding.chain_key.try_into()?,
            };

            if !sig.verify() {
                return Err(CoreEthereumIndexerError::AccountEntrySignatureVerification);
            }

            db.update_account_and_snapshot(&updated_account, snapshot).await?;

            updated_account
        }
        HoprAnnouncementsEvents::RevokeAnnouncementFilter(revocation) => {
            let maybe_account = db.get_account(&revocation.node.try_into()?).await?;

            if let Some(mut account) = maybe_account {
                account.update(AccountType::NotAnnounced);
                db.update_account_and_snapshot(&account, snapshot).await?;

                account
            } else {
                return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding);
            }
        }
    })
}

#[cfg(test)]
pub mod tests {
    use async_std;
    use bindings::hopr_announcements::{AddressAnnouncementFilter, KeyBindingFilter, RevokeAnnouncementFilter};
    use core_crypto::{
        keypairs::{Keypair, OffchainKeypair},
        types::{OffchainPublicKey, OffchainSignature},
    };
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::account::{AccountEntry, AccountSignature, AccountType};
    use ethers::{abi::{RawLog, encode, Token, Address as EthereumAddress}, prelude::*};
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Snapshot},
        traits::BinarySerializable,
    };

    const SELF_PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const SELF_CHAIN_ADDRESS: [u8; 20] = hex!("2e505638d318598334c0a2c2e887e0ff1a23ec6a");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::random(),
        )
    }

    #[async_std::test]
    async fn announce_workflow() {
        let mut db = create_mock_db();

        let keypair = OffchainKeypair::from_secret(&SELF_PRIV_KEY).unwrap();
        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        let sig = AccountSignature::new(&keypair, chain_key);

        let mut keybinding_payload = Vec::with_capacity(OffchainSignature::SIZE + OffchainPublicKey::SIZE + 32);
        keybinding_payload.extend_from_slice(&sig.signature.to_bytes());
        keybinding_payload.extend_from_slice(&sig.pub_key.to_bytes());
        // pad the chain_key address
        keybinding_payload.extend_from_slice(&[0u8; 12]);
        keybinding_payload.extend_from_slice(&chain_key.to_bytes());

        let keybinding_log = RawLog {
            topics: vec![KeyBindingFilter::signature()],
            data: encode(&[Token::FixedBytes(Vec::from(sig.signature.to_bytes())), Token::FixedBytes(Vec::from(sig.pub_key.to_bytes())), Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes()))]),
        };

        let account_entry = AccountEntry::new(keypair.public().clone(), chain_key, AccountType::NotAnnounced);

        assert_eq!(
            super::on_announce(&mut db, &keybinding_log, 0u32, &Snapshot::default())
                .await
                .unwrap(),
            account_entry
        );

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);

        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        let address_announcement_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes())), Token::String(test_multiaddr.to_string())]),
        };

        let announced_account_entry = AccountEntry::new(keypair.public().clone(), chain_key, AccountType::Announced { multiaddr: test_multiaddr, updated_block: 0 });

        assert_eq!(super::on_announce(&mut db, &address_announcement_log, 0u32, &Snapshot::default()).await.unwrap(), announced_account_entry);

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), announced_account_entry);

        let revoke_announcement_log = RawLog {
            topics: vec![RevokeAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes()))]),
        };

        assert_eq!(super::on_announce(&mut db, &revoke_announcement_log, 0u32, &Snapshot::default()).await.unwrap(), account_entry);

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);
    }
}
#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use core_types::account::AccountEntry;
    use ethers::core::abi::RawLog;
    use ethers::types::H256;
    use hex::decode_to_slice;
    use js_sys::{Array, Uint8Array};
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::Snapshot;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures;

    #[wasm_bindgen]
    pub async fn on_announcement_event(
        db: &Database,
        topics: Array,
        data: &str,
        block_number: &str,
        snapshot: &Snapshot,
    ) -> JsResult<AccountEntry> {
        let mut decoded_data = Vec::with_capacity(data.len() * 2);
        ok_or_jserr!(decode_to_slice(data, &mut decoded_data))?;

        let val = db.as_ref_counted();
        let mut g = val.write().await;

        let mut decoded_topics: Vec<H256> = vec![];

        for topic in topics.iter() {
            let mut decoded: [u8; 32] = [0u8; 32];

            ok_or_jserr!(decode_to_slice(
                Uint8Array::from(topic.to_owned()).to_vec(),
                &mut decoded
            ))?;

            decoded_topics.push(decoded.to_owned().into());
        }

        super::on_announce(
            &mut *g,
            &RawLog {
                topics: decoded_topics,
                data: decoded_data,
            },
            u32::from_str_radix(block_number, 10).map_err(|e| JsValue::from(e.to_string()))?,
            snapshot,
        )
        .await
        .map_err(|e| JsValue::from(e.to_string()))
    }
}
