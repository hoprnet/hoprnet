use crate::errors::CoreEthereumError::{CommitmentError, DbError};
use crate::errors::Result;
use async_trait::async_trait;
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

/// Trait for retrieving and setting the commitment information from the chain
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait ChainCommitter {
    async fn get_commitment(&self) -> Option<Hash>;
    async fn set_commitment(&mut self, commitment: &Hash) -> String;
}

async fn create_commitment_chain<T, C>(
    db: &mut T,
    channel_id: &Hash,
    initial_commitment_seed: &[u8],
    committer: &mut C,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
    C: ChainCommitter,
{
    let intermediates = iterate_hash(initial_commitment_seed, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE);

    db.store_hash_intermediaries(channel_id, &intermediates).await?;
    let current = Hash::new(&intermediates.hash);
    db.set_current_commitment(channel_id, &current)
        .then(|_| committer.set_commitment(&current))
        .await;

    info!("commitment chain initialized for {channel_id}");
    Ok(())
}

/// Holds the commitment information of a specific channel.
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

impl ChannelCommitmentInfo {
    /// Generate the initial commitment seed using this channel information and the given
    ///node private key.
    pub fn create_initial_commitment_seed(&self, private_key: &[u8]) -> Result<Box<[u8]>> {
        let mut buf = Vec::with_capacity(U256::SIZE + 4 + Hash::SIZE + self.contract_address.len());
        buf.extend_from_slice(&self.channel_epoch.to_bytes());
        buf.extend_from_slice(&self.chain_id.to_be_bytes());
        buf.extend_from_slice(&self.channel_id.to_bytes());
        buf.extend_from_slice(self.contract_address.as_bytes());

        Ok(derive_commitment_seed(private_key, &buf).into())
    }
}

/// Initializes commitment for the given channel.
/// The ChainCommitter is used to tell the current state of the channel and to determine if re-initialization is
/// needed or not.
pub async fn initialize_commitment<T, C>(
    db: &mut T,
    private_key: &[u8],
    channel_info: &ChannelCommitmentInfo,
    committer: &mut C,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
    C: ChainCommitter,
{
    let contains_already = db.get_commitment(&channel_info.channel_id, 0).await?.is_some();
    let chain_commitment = committer.get_commitment().await;
    if contains_already && chain_commitment.is_some() {
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
    use crate::commitment::{
        bump_commitment, find_commitment_preimage, initialize_commitment, ChannelCommitmentInfo, MockChainCommitter,
    };
    use async_std;
    use core_crypto::types::{Hash, PublicKey};
    use core_ethereum_db::db::CoreEthereumDb;
    use hex_literal::hex;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::U256;
    use utils_types::traits::BinarySerializable;

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
        let comm_info = ChannelCommitmentInfo {
            chain_id: 1,
            contract_address: "fake_address".to_string(),
            channel_id: Hash::new(&[0u8; Hash::SIZE]),
            channel_epoch: U256::zero(),
        };
        let mut db = create_mock_db();

        let mut committer = MockChainCommitter::new();
        committer.expect_get_commitment().times(1).return_const(None);
        committer
            .expect_set_commitment()
            .times(1)
            .return_const(comm_info.channel_id.to_string());

        initialize_commitment(&mut db, &PRIV_KEY, &comm_info, &mut committer)
            .await
            .unwrap();

        let c1 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();

        bump_commitment(&mut db, &comm_info.channel_id, &c1).await.unwrap();

        let c2 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();
        assert_eq!(c2.hash(), c1, "c2 is commitment of c1");

        committer.expect_get_commitment().times(1).return_const(Some(c2));
        initialize_commitment(&mut db, &PRIV_KEY, &comm_info, &mut committer)
            .await
            .unwrap();

        let c3 = find_commitment_preimage(&mut db, &comm_info.channel_id).await.unwrap();
        assert_eq!(c2, c3, "repeated initializations should return the same commitment");
    }
}
