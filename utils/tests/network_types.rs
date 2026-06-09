#![cfg(feature = "network-types")]

use hopr_utils::network_types::{IpOrHost, SealedHost};

#[test]
fn network_types_feature_exposes_sealed_host() -> anyhow::Result<()> {
    let host: IpOrHost = "127.0.0.1:1000".parse()?;
    let sealed: SealedHost = host.clone().into();

    assert_eq!(host, sealed.try_into()?);
    Ok(())
}
