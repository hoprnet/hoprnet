use hopr_api::chain::ChainReadAccountOperations;
use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
use hopr_internal_types::{
    channels::ChannelStatus,
    prelude::{AccountEntry, AccountType, ChannelEntry},
};
use hopr_primitive_types::{
    balance::WxHOPR,
    prelude::{Address, BytesRepresentable, HoprBalance, XDai, XDaiBalance},
};

lazy_static::lazy_static! {
    static ref CHAIN_KEYS: Vec<ChainKeypair> = (0..3).map(|_| ChainKeypair::random()).collect();
    static ref OFFCHAIN_KEYS: Vec<OffchainKeypair> = (0..3).map(|_| OffchainKeypair::random()).collect();

    static ref ACCOUNTS: Vec<AccountEntry> = vec![
        AccountEntry {
            public_key: *OFFCHAIN_KEYS[0].public(),
            chain_addr: CHAIN_KEYS[0].public().to_address(),
            entry_type: AccountType::Announced(vec!["/ip4/34.65.237.196/udp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk".parse().unwrap()]),
            safe_address: None,
            key_id: 1_u32.into(),
        },
        AccountEntry {
            public_key: *OFFCHAIN_KEYS[1].public(),
            chain_addr: CHAIN_KEYS[1].public().to_address(),
            entry_type: AccountType::Announced(vec!["/ip4/34.65.237.190/udp/9091/p2p/12D3KooWPGsW7vZ8VsmJ9Lws9vsKaBiACZXQ3omRm3rFUho5BpvF".parse().unwrap()]),
            safe_address: None,
            key_id: 2_u32.into(),
        },
        AccountEntry {
            public_key: *OFFCHAIN_KEYS[2].public(),
            chain_addr: CHAIN_KEYS[2].public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 3_u32.into(),
        }
    ];
    static ref CHANNELS: Vec<ChannelEntry> = vec![
        ChannelEntry::new(
            CHAIN_KEYS[0].public().to_address(),
            CHAIN_KEYS[1].public().to_address(),
            100_u32.into(),
            1_u64.into(),
            ChannelStatus::Open,
            1_u32.into(),
        ),
        ChannelEntry::new(
            CHAIN_KEYS[2].public().to_address(),
            CHAIN_KEYS[0].public().to_address(),
            200_u32.into(),
            1_u64.into(),
            ChannelStatus::Open,
            2_u32.into(),
        ),
        ChannelEntry::new(
            CHAIN_KEYS[1].public().to_address(),
            CHAIN_KEYS[0].public().to_address(),
            10_u32.into(),
            3_u64.into(),
            ChannelStatus::Open,
            1_u32.into(),
        )
    ];
}

#[tokio::test]
async fn hopr_block_chain_connector_should_return_channels() -> anyhow::Result<()> {
    let mock_client = BlokliTestStateBuilder::default()
        .with_accounts(
            ACCOUNTS
                .iter()
                .cloned()
                .map(|a| (a, HoprBalance::zero(), XDaiBalance::zero())),
        )
        .with_channels(CHANNELS.iter().cloned())
        .with_balances::<WxHOPR>([
            (CHAIN_KEYS[0].public().to_address(), 1000_u32.into()),
            (CHAIN_KEYS[1].public().to_address(), 2000_u32.into()),
        ])
        .with_balances::<XDai>([
            (CHAIN_KEYS[0].public().to_address(), 1_u32.into()),
            (CHAIN_KEYS[1].public().to_address(), 2_u32.into()),
        ])
        .build_static_client();

    let me = ChainKeypair::random();

    let mut connector = create_trustful_hopr_blokli_connector(
        &me,
        Default::default(),
        mock_client,
        Address::new(&[2u8; Address::SIZE]),
    )
    .await?;

    connector.connect().await?;

    assert_eq!(
        HoprBalance::from(1000_u32),
        connector.get_balance(&CHAIN_KEYS[0]).await?
    );
    assert_eq!(XDaiBalance::from(1_u32), connector.get_balance(&CHAIN_KEYS[0]).await?);

    Ok(())
}
