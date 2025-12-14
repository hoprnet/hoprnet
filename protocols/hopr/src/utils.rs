use std::sync::Arc;

use hex_literal::hex;
use hopr_chain_connector::{
    HoprBlockchainSafeConnector, create_trustful_hopr_blokli_connector,
    testing::{BlokliTestClient, BlokliTestStateBuilder, StaticState},
};
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

lazy_static::lazy_static! {
    pub static ref PEERS: [(ChainKeypair, OffchainKeypair); 5] = [
        (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
        (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
        (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
        (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed")),
        (hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"), hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")),
    ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));
}

#[derive(Clone)]
pub struct Node {
    pub chain_key: ChainKeypair,
    pub offchain_key: OffchainKeypair,
    pub node_db: HoprNodeDb,
    pub chain_api: Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
}

pub fn create_blokli_client() -> anyhow::Result<BlokliTestClient<StaticState>> {
    Ok(BlokliTestStateBuilder::default()
        .with_accounts(PEERS.iter().enumerate().map(|(i, (chain_key, offchain_key))| {
            (
                AccountEntry {
                    public_key: *offchain_key.public(),
                    chain_addr: chain_key.public().to_address(),
                    entry_type: AccountType::NotAnnounced,
                    safe_address: Some([(i + 10) as u8; 20].into()),
                    key_id: ((i + 1) as u32).into(),
                },
                HoprBalance::new_base(100),
                XDaiBalance::new_base(1),
            )
        }))
        .with_channels(
            PEERS
                .iter()
                .enumerate()
                .map(|(i, (chain_key, _))| {
                    ChannelEntry::new(
                        chain_key.public().to_address(),
                        PEERS[(i + 1) % PEERS.len()].0.public().to_address(),
                        HoprBalance::new_base(100),
                        0,
                        ChannelStatus::Open,
                        1,
                    )
                })
                .chain(PEERS.iter().enumerate().rev().map(|(i, (chain_key, _))| {
                    ChannelEntry::new(
                        chain_key.public().to_address(),
                        PEERS[if i > 0 { i - 1 } else { PEERS.len() - 1 }]
                            .0
                            .public()
                            .to_address(),
                        HoprBalance::new_base(100),
                        0,
                        ChannelStatus::Open,
                        1,
                    )
                })),
        )
        .build_static_client())
}

pub async fn create_node(index: usize, blokli_client: &BlokliTestClient<StaticState>) -> anyhow::Result<Node> {
    if index >= PEERS.len() {
        return Err(anyhow::anyhow!("invalid index"));
    }

    let mut chain_api = create_trustful_hopr_blokli_connector(
        &PEERS[index].0,
        Default::default(),
        blokli_client.clone(),
        [10u8; 20].into(),
    )
    .await?;
    chain_api.connect().await?;

    Ok(Node {
        chain_key: PEERS[index].0.clone(),
        offchain_key: PEERS[index].1.clone(),
        node_db: HoprNodeDb::new_in_memory().await?,
        chain_api: Arc::new(chain_api),
    })
}
