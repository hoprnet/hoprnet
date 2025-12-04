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
    use hopr_chain_connector::{
        HoprBlockchainSafeConnector, create_trustful_hopr_blokli_connector,
        testing::{BlokliTestClient, BlokliTestStateBuilder, StaticState},
    };
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

    lazy_static::lazy_static! {
        static ref PEERS: [(ChainKeypair, OffchainKeypair); 5] = [
            (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
            (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
            (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
            (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed")),
            (hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"), hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")),
        ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));
    }

    struct Node {
        pub chain_key: ChainKeypair,
        pub offchain_key: OffchainKeypair,
        pub node_db: HoprNodeDb,
        pub chain_api: Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
    }

    fn create_blokli_client() -> anyhow::Result<BlokliTestClient<StaticState>> {
        Ok(BlokliTestStateBuilder::default()
            .with_accounts(PEERS.iter().enumerate().map(|(i, (chain_key, offchain_key))| {
                (
                    AccountEntry {
                        public_key: *offchain_key.public(),
                        chain_addr: chain_key.public().to_address(),
                        entry_type: AccountType::NotAnnounced,
                        safe_address: Some([1u8; 20].into()),
                        key_id: ((i + 1) as u32).into(),
                    },
                    HoprBalance::new_base(100),
                    XDaiBalance::new_base(1),
                )
            }))
            .with_channels(PEERS.iter().enumerate().map(|(i, (chain_key, _))| {
                ChannelEntry::new(
                    chain_key.public().to_address(),
                    PEERS[(i + 1) % PEERS.len()].0.public().to_address(),
                    HoprBalance::new_base(100),
                    0,
                    ChannelStatus::Open,
                    1,
                )
            }))
            .build_static_client())
    }

    async fn create_node(index: usize, blokli_client: &BlokliTestClient<StaticState>) -> anyhow::Result<Node> {
        let mut chain_api = create_trustful_hopr_blokli_connector(
            &PEERS[index].0,
            Default::default(),
            blokli_client.clone(),
            [10u8; 20].into(),
        )
        .await?;
        chain_api.connect(std::time::Duration::from_secs(1)).await?;

        Ok(Node {
            chain_key: PEERS[index].0.clone(),
            offchain_key: PEERS[index].1.clone(),
            node_db: HoprNodeDb::new_in_memory().await?,
            chain_api: Arc::new(chain_api),
        })
    }

    type TestEncoder = HoprEncoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        MemorySurbStore,
        HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
    >;

    type TestDecoder = HoprDecoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        MemorySurbStore,
        HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
    >;

    fn create_encoder(sender: &Node) -> TestEncoder {
        HoprEncoder::new(
            sender.chain_key.clone(),
            sender.chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                sender.chain_api.clone(),
                sender.node_db.clone(),
                sender.chain_key.clone(),
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        )
    }

    fn create_decoder(receiver: &Node) -> TestDecoder {
        HoprDecoder::new(
            (receiver.offchain_key.clone(), receiver.chain_key.clone()),
            receiver.chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                receiver.chain_api.clone(),
                receiver.node_db.clone(),
                receiver.chain_key.clone(),
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        )
    }

    #[tokio::test]
    async fn encode_decode_packet() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let data = b"some random message to encode and decode";

        let out_packet = encoder
            .encode_packet(
                data,
                ResolvedTransportRouting::Forward {
                    pseudonym: HoprPseudonym::random(),
                    forward_path: ValidatedPath::direct(
                        *receiver.offchain_key.public(),
                        receiver.chain_key.public().to_address(),
                    ),
                    return_paths: vec![],
                },
                None,
            )
            .await?;

        let in_packet = decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_packet_should_fail_for_too_long_messages() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);

        let data = hopr_crypto_random::random_bytes::<2048>();

        assert!(
            encoder
                .encode_packet(
                    data,
                    ResolvedTransportRouting::Forward {
                        pseudonym: HoprPseudonym::random(),
                        forward_path: ValidatedPath::direct(
                            *receiver.offchain_key.public(),
                            receiver.chain_key.public().to_address(),
                        ),
                        return_paths: vec![],
                    },
                    None,
                )
                .await
                .is_err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_packet_on_relay() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let relay = create_node(1, &blokli_client).await?;
        let receiver = create_node(2, &blokli_client).await?;

        let sender_encoder = create_encoder(&sender);
        let relay_decoder = create_decoder(&relay);
        let receiver_decoder = create_decoder(&receiver);

        let data = b"some random message to encode and decode";

        let out_packet = sender_encoder
            .encode_packet(
                data,
                ResolvedTransportRouting::Forward {
                    pseudonym: HoprPseudonym::random(),
                    forward_path: ValidatedPath::new(
                        sender.chain_key.public().to_address(),
                        vec![
                            relay.chain_key.public().to_address(),
                            receiver.chain_key.public().to_address(),
                        ],
                        &sender.chain_api.as_path_resolver(),
                    )
                    .await?,
                    return_paths: vec![],
                },
                None,
            )
            .await?;

        let fwd_packet = relay_decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
        let fwd_packet = fwd_packet
            .try_as_forwarded()
            .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

        let in_packet = receiver_decoder
            .decode(relay.offchain_key.public().into(), fwd_packet.data)
            .await?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_acknowledgements() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let acks = (0..10)
            .map(|_| VerifiedAcknowledgement::random(&PEERS[0].1))
            .collect::<Vec<_>>();
        let out_packet = encoder
            .encode_acknowledgements(acks.clone(), PEERS[1].1.public())
            .await?;

        let in_packet = decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
        let in_packet = in_packet
            .try_as_acknowledgement()
            .ok_or(anyhow::anyhow!("packet is not acknowledgement"))?;

        assert_eq!(acks.len(), in_packet.received_acks.len());

        for (i, ack) in in_packet.received_acks.into_iter().enumerate() {
            let verified = ack.verify(PEERS[0].1.public())?;
            assert_eq!(acks[i], verified);
        }

        Ok(())
    }

    #[tokio::test]
    async fn encode_should_fail_on_too_many_acknowledgements() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let acks = (0..1000)
            .map(|_| VerifiedAcknowledgement::random(&PEERS[0].1))
            .collect::<Vec<_>>();

        assert!(
            encoder
                .encode_acknowledgements(acks.clone(), PEERS[1].1.public())
                .await
                .is_err()
        );

        Ok(())
    }
}
