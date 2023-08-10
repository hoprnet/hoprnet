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
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::primitives::Address;

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::random(),
        )
    }

    #[test]
    fn announce_workflow() {
        let mut db = create_mock_db();
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
