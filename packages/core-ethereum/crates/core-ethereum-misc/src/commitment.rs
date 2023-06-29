use crate::errors::CoreEthereumError::{CommitmentError, CryptoError, DbError};
use crate::errors::Result;
use core_crypto::derivation::derive_commitment_seed;
use core_crypto::iterated_hash::{iterate_hash, recover_iterated_hash};
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use futures::FutureExt;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::U256;
use utils_types::traits::{BinarySerializable, ToHex};

const DB_ITERATION_BLOCK_SIZE: usize = 10000;
const TOTAL_ITERATIONS: usize = 100000;

/// Retrieves commitment pre-image for the given channel ID from the database.
/// Returns `CalculationError` if commitment couldn't be found.
pub async fn find_commitment_preimage<T: HoprCoreEthereumDbActions>(db: &T, channel_id: &Hash) -> Result<Hash> {
    let current_commitment = db.get_current_commitment(channel_id).await?;
    let recovered = recover_iterated_hash(
        &current_commitment
            .ok_or(CommitmentError("no current commitment".to_string()))?
            .to_bytes(),
        &|iteration: usize| async move {
            match db.get_commitment(channel_id, iteration).await {
                Ok(opt) => opt.map(|hash| hash.to_bytes()),
                Err(e) => {
                    error!("failed to retrieve iteration {channel_id} #{iteration}: {e}");
                    None
                }
            }
        },
        TOTAL_ITERATIONS,
        DB_ITERATION_BLOCK_SIZE,
        None,
    )
    .await?;

    Ok(Hash::new(&recovered.intermediate))
}

/// Updates the commitment in the database on the given channel ID with the new value.
pub async fn bump_commitment<T: HoprCoreEthereumDbActions>(
    db: &mut T,
    channel_id: &Hash,
    new_commitment: &Hash,
) -> Result<()> {
    db.set_current_commitment(channel_id, new_commitment)
        .await
        .map_err(|e| DbError(e))
}

async fn create_commitment_chain<T, C, F>(
    db: &mut T,
    channel_id: &Hash,
    initial_commitment_seed: &[u8],
    committer: C,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
    C: Fn(Hash) -> F,
    F: futures::Future<Output = Option<String>>,
{
    let intermediates = iterate_hash(initial_commitment_seed, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE);

    db.store_hash_intermediaries(channel_id, &intermediates).await?;
    let current = Hash::new(&intermediates.hash);
    db.set_current_commitment(channel_id, &current)
        .then(|_| committer(current))
        .await;

    info!("commitment chain initialized for {channel_id}");
    Ok(())
}

/// Holds the commitment information of a specific channel.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug)]
pub struct ChannelCommitmentInfo {
    /// ID of the blockchain network
    pub chain_id: u32,
    /// Channel contract address
    pub contract_address: String,
    /// ID of the channel
    pub channel_id: Hash,
    /// Channel epoch value
    pub channel_epoch: U256,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl ChannelCommitmentInfo {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(chain_id: u32, contract_address: String, channel_id: Hash, channel_epoch: U256) -> Self {
        Self {
            chain_id,
            contract_address,
            channel_id,
            channel_epoch,
        }
    }
}

impl ChannelCommitmentInfo {
    /// Generate the initial commitment seed using this channel information and the given
    ///node private key.
    pub fn create_initial_commitment_seed(&self, private_key: &[u8]) -> Result<Box<[u8]>> {
        let mut buf = Vec::with_capacity(U256::SIZE + 4 + Hash::SIZE + self.contract_address.len());
        buf.extend_from_slice(&self.channel_epoch.to_bytes());
        buf.extend_from_slice(&self.chain_id.to_be_bytes());
        buf.extend_from_slice(&self.channel_id.to_bytes());
        buf.extend_from_slice(self.contract_address.as_bytes());

        derive_commitment_seed(private_key, &buf).map_err(|e| CryptoError(e))
    }
}

/// Initializes commitment for the given channel.
/// The ChainCommitter is used to tell the current state of the channel and to determine if re-initialization is
/// needed or not.
pub async fn initialize_commitment<T, C, F>(
    db: &mut T,
    private_key: &[u8],
    channel_info: &ChannelCommitmentInfo,
    committer: C,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
    C: Fn(Hash) -> F,
    F: futures::Future<Output = Option<String>>,
{
    let contains_already = db.get_commitment(&channel_info.channel_id, 0).await?.is_some();
    let chain_commitment = db.get_channel(&channel_info.channel_id).await?.map(|c| c.commitment);

    if contains_already && chain_commitment.is_some() {
        debug!("commitment already present for channel {}", channel_info.channel_id);
        match find_commitment_preimage(db, &channel_info.channel_id).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                warn!("Secret is found but failed to find preimage, reinitializing.. {e}")
            }
        }
    }

    debug!(
        "reinitializing (db: {contains_already}, chain: {})",
        chain_commitment.map(|h| h.to_hex()).unwrap_or("false".to_string())
    );

    create_commitment_chain(
        db,
        &channel_info.channel_id,
        &channel_info.create_initial_commitment_seed(private_key)?,
        committer,
    )
    .await
}

#[cfg(all(not(target_arch = "wasm32"), test))]
mod tests {
    use crate::commitment::{bump_commitment, find_commitment_preimage, initialize_commitment, ChannelCommitmentInfo};
    use async_std;
    use core_crypto::types::PublicKey;
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use hex_literal::hex;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::BalanceType::HOPR;
    use utils_types::primitives::{Balance, Snapshot, U256};

    const PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            PublicKey::from_privkey(&PRIV_KEY).unwrap(),
        )
    }

    #[async_std::test]
    async fn test_should_publish_hash_secret() {
        env_logger::init();

        let channel = ChannelEntry {
            source: PublicKey::random(),
            destination: PublicKey::random(),
            balance: Balance::zero(HOPR),
            commitment: Default::default(),
            ticket_epoch: U256::zero(),
            ticket_index: U256::zero(),
            status: ChannelStatus::Open,
            channel_epoch: U256::zero(),
            closure_time: U256::zero(),
        };

        let comm_info = ChannelCommitmentInfo {
            chain_id: 1,
            contract_address: "fake_address".to_string(),
            channel_id: channel.get_id(),
            channel_epoch: U256::zero(),
        };
        let mut db = create_mock_db();

        let committer = |_| async { Some(comm_info.channel_id.to_string()) };

        initialize_commitment(&mut db, &PRIV_KEY, &comm_info, committer)
            .await
            .unwrap();

        let c1 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();

        bump_commitment(&mut db, &comm_info.channel_id, &c1).await.unwrap();

        let c2 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();
        assert_eq!(c2.hash(), c1, "c2 is commitment of c1");

        db.update_channel_and_snapshot(&comm_info.channel_id, &channel, &Snapshot::default())
            .await
            .unwrap();

        initialize_commitment(&mut db, &PRIV_KEY, &comm_info, |_| async { None })
            .await
            .unwrap();

        let c3 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();
        assert_eq!(c2, c3, "repeated initializations should return the same commitment");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use js_sys::Uint8Array;
    use utils_log::error;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    use crate::commitment::ChannelCommitmentInfo;

    #[wasm_bindgen]
    pub async fn initialize_commitment(
        db: &Database,
        private_key: &[u8],
        channel_info: &ChannelCommitmentInfo,
        set_commitment: &js_sys::Function, // async (Uint8Array) => String
    ) -> JsResult<()> {
        let val = db.as_ref_counted();
        //let r = {
        let mut g = val.write().await;
        //console_log!("++++ initializing commitment preimage");
        ok_or_jserr!(
            super::initialize_commitment(&mut *g, private_key, channel_info, |commitment: Hash| async move {
                let this = JsValue::null();
                let hash: JsValue = Uint8Array::from(commitment.to_bytes().as_ref()).into();
                match set_commitment.call1(&this, &hash) {
                    Ok(r) => {
                        let promise = js_sys::Promise::from(r);
                        wasm_bindgen_futures::JsFuture::from(promise)
                            .await
                            .map_err(|e| error!("could not set commitment {:?}", e.as_string()))
                            .map(|v| v.as_string().unwrap())
                            .ok()
                    }
                    Err(e) => {
                        error!("not call set commitment {:?}", e.as_string());
                        None
                    }
                }
            })
            .await
        )
        //};
        //console_log!("==== initializing commitment preimage");
        //r
    }

    #[wasm_bindgen]
    pub async fn find_commitment_preimage(db: &Database, channel_id: &Hash) -> JsResult<Hash> {
        let val = db.as_ref_counted();
        let mut g = val.write().await;
        ok_or_jserr!(super::find_commitment_preimage(&mut *g, channel_id).await)
    }

    #[wasm_bindgen]
    pub async fn bump_commitment(db: &Database, channel_id: &Hash, new_commitment: &Hash) -> JsResult<()> {
        let val = db.as_ref_counted();
        let mut g = val.write().await;
        ok_or_jserr!(super::bump_commitment(&mut *g, channel_id, new_commitment).await)
    }
}
