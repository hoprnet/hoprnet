use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
use hopr_transport_probe::{Probe, ProbeConfig as Config};

lazy_static::lazy_static!(
    static ref OFFCHAIN_KEYPAIR: OffchainKeypair = OffchainKeypair::random();
    static ref ONCHAIN_KEYPAIR: ChainKeypair = ChainKeypair::random();
);

#[tokio::test]
async fn probe_should_timeout_if_no_response_arrives_back() -> anyhow::Result<()> {
    let mut cfg: Config = Default::default();
    cfg.timeout = std::time::Duration::from_millis(2);

    let probe = Probe::new((*OFFCHAIN_KEYPAIR.public(), ONCHAIN_KEYPAIR.public().to_address()), cfg);

    Ok(())
}
