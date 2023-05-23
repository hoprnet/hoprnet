use async_trait::async_trait;
use core_crypto::iterated_hash::{iterate_hash, recover_iterated_hash};
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use futures::FutureExt;
use core_crypto::derivation::derive_commitment_seed;
use utils_log::{debug, warn};
use utils_types::primitives::U256;
use utils_types::traits::{BinarySerializable, ToHex};
use crate::errors::CoreEthereumError::{CommitmentError, CryptoError, DbError};
use crate::errors::Result;

pub const DB_ITERATION_BLOCK_SIZE: usize = 10000;
pub const TOTAL_ITERATIONS: usize = 100000;

pub async fn find_commitment_preimage<T: HoprCoreEthereumDbActions>(db: &T, channel_id: &Hash) -> Result<Hash> {
    let current_commitment = db.get_current_commitment(channel_id).await?;
    let recovered = recover_iterated_hash(
        &current_commitment.ok_or(CommitmentError("no current commitment".to_string()))?.to_bytes(),
        &|iteration: usize| async move {
            match db.get_commitment(channel_id, iteration).await {
                Ok(opt) => opt.map(|hash| hash.to_bytes()),
                Err(_) => {
                    // TODO: log error
                    None
                }
            }
        },
        TOTAL_ITERATIONS,
        DB_ITERATION_BLOCK_SIZE,
        None
    )
    .await?;

    Ok(Hash::new(&recovered.intermediate))
}

pub async fn bump_commitment<T: HoprCoreEthereumDbActions>(db: &mut T, channel_id: &Hash, new_commitment: &Hash) -> Result<()> {
    db.set_current_commitment(channel_id, new_commitment).await.map_err(|e| DbError(e))
}

#[async_trait(? Send)]
pub trait ChannelCommitter {
    async fn get_commitment(&self) -> Option<Hash>;
    async fn set_commitment(&self, commitment: &Hash) -> String;
}

pub async fn create_commitment_chain<T, C>(db: &mut T, channel_id: &Hash, initial_commitment_seed: &[u8], committer: &C) -> Result<()>
where T: HoprCoreEthereumDbActions, C: ChannelCommitter {
    let intermediates = iterate_hash(initial_commitment_seed, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE);

    db.store_hash_intermediaries(channel_id, &intermediates).await?;
    let current = Hash::new(&intermediates.hash);
    db.set_current_commitment(channel_id, &current)
        .then(|_| committer.set_commitment(&current))
        .await;

    Ok(())
}

pub struct ChannelCommitmentInfo {
    pub chain_id: u32,
    pub contract_address: String,
    pub channel_id: Hash,
    pub channel_epoch: U256
}

impl ChannelCommitmentInfo {
    /// Generate the initial commitment seed using this channel information and the given
    //  private node key.
    pub fn create_initial_commitment_seed(&self, private_key: &[u8]) -> Result<Box<[u8]>> {
        let mut buf = Vec::with_capacity(U256::SIZE + 4 + Hash::SIZE + self.contract_address.len());
        buf.extend_from_slice(&self.channel_epoch.to_bytes());
        buf.extend_from_slice(&self.chain_id.to_be_bytes());
        buf.extend_from_slice(&self.channel_id.to_bytes());
        buf.extend_from_slice(self.contract_address.as_bytes());

        derive_commitment_seed(private_key, &buf).map_err(|e| CryptoError(e))
    }
}

pub async fn initialize_commitment<T, C>(db: &mut T, private_key: &[u8], channel_info: &ChannelCommitmentInfo, committer: &C) -> Result<()>
where T: HoprCoreEthereumDbActions, C: ChannelCommitter {
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

    debug!("reinitializing (db: {contains_already}, chain: {})",
        chain_commitment.map(|h| h.to_hex()).unwrap_or("false".to_string()));

    create_commitment_chain(
        db,
        &channel_info.channel_id,
        &channel_info.create_initial_commitment_seed(private_key)?,
        committer
    ).await
}
