mod decoder;
mod encoder;

pub use decoder::HoprDecoder;
pub use encoder::HoprEncoder;

fn default_outgoing_win_prob() -> Option<hopr_internal_types::prelude::WinningProbability> {
    Some(hopr_internal_types::prelude::WinningProbability::ALWAYS)
}

/// Configuration of [`HoprEncoder`] and [`HoprDecoder`].
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Clone, Copy, Debug, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprCodecConfig {
    /// Optional price of outgoing tickets.
    ///
    /// If not set, the network default will be used, which is the minimum allowed ticket price in the HOPR network.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    pub outgoing_ticket_price: Option<hopr_primitive_types::balance::HoprBalance>,
    /// Optional probability of winning an outgoing ticket.
    ///
    /// If not set, the network default will be used, which is the minimum allowed winning probability in the HOPR
    /// network.
    ///
    /// The default is [`WinningProbability::ALWAYS`](hopr_internal_types::prelude::WinningProbability::ALWAYS).
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_outgoing_win_prob"),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    #[default(default_outgoing_win_prob())]
    pub outgoing_win_prob: Option<hopr_internal_types::prelude::WinningProbability>,
}

impl PartialEq for HoprCodecConfig {
    fn eq(&self, other: &Self) -> bool {
        self.outgoing_ticket_price.eq(&other.outgoing_ticket_price)
            && match (self.outgoing_win_prob, other.outgoing_win_prob) {
                (Some(a), Some(b)) => a.approx_eq(&b),
                (None, None) => true,
                _ => false,
            }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hex_literal::hex;
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_db_node::HoprNodeDb;
    use hopr_internal_types::prelude::*;
    use hopr_network_types::prelude::ResolvedTransportRouting;
    use hopr_primitive_types::prelude::*;

    use super::*;
    use crate::{
        HoprTicketProcessor, HoprTicketProcessorConfig, MemorySurbStore, PacketDecoder, PacketEncoder, SurbStoreConfig,
    };

    pub const PRIVATE_KEY_1: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    pub const PRIVATE_KEY_2: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");

    #[tokio::test]
    async fn encode_decode_packet() -> anyhow::Result<()> {
        let sender_chain_key = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let receiver_chain_key = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;
        let sender_offchain_key = OffchainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let receiver_offchain_key = OffchainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (
                    AccountEntry {
                        public_key: *sender_offchain_key.public(),
                        chain_addr: sender_chain_key.public().to_address(),
                        entry_type: AccountType::NotAnnounced,
                        safe_address: Some([1u8; 20].into()),
                        key_id: 1.into(),
                    },
                    HoprBalance::new_base(100),
                    XDaiBalance::new_base(1),
                ),
                (
                    AccountEntry {
                        public_key: *receiver_offchain_key.public(),
                        chain_addr: receiver_chain_key.public().to_address(),
                        entry_type: AccountType::NotAnnounced,
                        safe_address: Some([2u8; 20].into()),
                        key_id: 2.into(),
                    },
                    HoprBalance::new_base(100),
                    XDaiBalance::new_base(1),
                ),
            ])
            .with_channels([ChannelEntry::new(
                sender_chain_key.public().to_address(),
                receiver_chain_key.public().to_address(),
                HoprBalance::new_base(100),
                0,
                ChannelStatus::Open,
                1,
            )])
            .build_static_client();

        let sender_node_db = HoprNodeDb::new_in_memory().await?;
        let mut sender_chain_api = create_trustful_hopr_blokli_connector(
            &sender_chain_key,
            Default::default(),
            blokli_client.clone(),
            [10u8; 20].into(),
        )
        .await?;
        sender_chain_api.connect(std::time::Duration::from_secs(1)).await?;
        let sender_chain_api = Arc::new(sender_chain_api);

        let receiver_node_db = HoprNodeDb::new_in_memory().await?;
        let mut receiver_chain_api = create_trustful_hopr_blokli_connector(
            &sender_chain_key,
            Default::default(),
            blokli_client.clone(),
            [10u8; 20].into(),
        )
        .await?;
        receiver_chain_api.connect(std::time::Duration::from_secs(1)).await?;
        let receiver_chain_api = Arc::new(receiver_chain_api);

        let encoder = HoprEncoder::new(
            sender_chain_key.clone(),
            sender_chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                sender_chain_api,
                sender_node_db,
                sender_chain_key,
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        );

        let decoder = HoprDecoder::new(
            (receiver_offchain_key.clone(), receiver_chain_key.clone()),
            receiver_chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                receiver_chain_api,
                receiver_node_db,
                receiver_chain_key.clone(),
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        );

        let data = b"some random message to encode and decode";

        let out_packet = encoder
            .encode_packet(
                data,
                ResolvedTransportRouting::Forward {
                    pseudonym: HoprPseudonym::random(),
                    forward_path: ValidatedPath::direct(
                        *receiver_offchain_key.public(),
                        receiver_chain_key.public().to_address(),
                    ),
                    return_paths: vec![],
                },
                None,
            )
            .await?;

        let in_packet = decoder.decode(out_packet.next_hop.into(), out_packet.data).await?;

        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
        Ok(())
    }
}
