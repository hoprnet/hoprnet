use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt};
use hopr_api::{
    chain::ChainValues,
    node::TicketEvent,
    types::{
        crypto::keypairs::Keypair,
        crypto_random::Randomizable,
        internal::{prelude::*, routing::ResolvedTransportRouting},
        primitive::prelude::HoprBalance,
    },
};
use hopr_chain_connector::create_trustful_hopr_blokli_connector;
use hopr_crypto_packet::{HoprPixSpec, HoprSurb, prelude::HoprPacket};
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
use hopr_protocol_hopr::{
    HoprCodecConfig, HoprDecoder, HoprEncoder, HoprUnacknowledgedTicketProcessor,
    HoprUnacknowledgedTicketProcessorConfig, MemorySurbStore, SurbStoreConfig,
};
use hopr_protocol_pix::{
    DEFAULT_POLY_THRESHOLD, EntryShareGenerator, MAX_POLYS_PER_SSA, SsaGeneratorConfig, SsaIndex, SsaShareGenerator,
};
use hopr_ticket_manager::{HoprTicketFactory, RedbStore};
use hopr_transport::testing::harness::{CHAIN_DATA, PEERS, PEERS_CHAIN, random_packets_of_count, resolve_mock_path};
use libp2p::PeerId;

const SAMPLE_SIZE: usize = 50;

/// Deployed polynomial threshold. `DEFAULT_PIX_SHARES_PER_POLY`
/// (`transport/session/src/types.rs`) is an alias of [`DEFAULT_POLY_THRESHOLD`], so this is the
/// negotiated value by construction.
///
/// This is what sets the per-share cost: `next_share` is a Horner evaluation over `threshold`
/// coefficients.
const PIX_THRESHOLD: u16 = DEFAULT_POLY_THRESHOLD;

/// Whether the PIX share generator handed to the encoder has a committed SSA.
///
/// Without one, `next_share` short-circuits to `Ok(None)` and no share is embedded into the
/// SURB, which is what a Session that does not use PIX costs. With one, every SURB the
/// encoder builds carries a real share, so the difference between the two ids in this group
/// is the in-situ PIX overhead of the sending pipeline.
#[derive(Debug, Clone, Copy)]
enum PixMode {
    Off,
    On,
}

impl PixMode {
    const ALL: [PixMode; 2] = [PixMode::Off, PixMode::On];

    fn as_str(&self) -> &'static str {
        match self {
            PixMode::Off => "pix_off",
            PixMode::On => "pix_on",
        }
    }

    /// Builds the generator and the pseudonym that packets must be sent under.
    ///
    /// `polynomials_per_ssa` is the maximum rather than the deployed 8192 only to widen the
    /// share budget — one SURB consumes one share and a run sends hundreds of thousands of
    /// them. The polynomial count does not affect the per-share cost; [`PIX_THRESHOLD`] does.
    fn build(&self) -> (Arc<SsaShareGenerator<HoprPixSpec>>, HoprPseudonym) {
        let pseudonym = HoprPseudonym::random();
        match self {
            PixMode::Off => (
                Arc::new(SsaShareGenerator::new(SsaGeneratorConfig::default())),
                pseudonym,
            ),
            PixMode::On => {
                let generator = SsaShareGenerator::new(SsaGeneratorConfig {
                    threshold: PIX_THRESHOLD,
                    polynomials_per_ssa: MAX_POLYS_PER_SSA,
                    ..Default::default()
                });
                generator
                    .new_ssa_commitment(&pseudonym, SsaIndex::MIN)
                    .expect("pix commitment must succeed");
                (Arc::new(generator), pseudonym)
            }
        }
    }
}

/// Fails loudly if a `pix_on` generator ran out of shares mid-run.
///
/// Once the budget is gone, `next_share` returns `Ok(None)`, SURBs stop carrying shares, and
/// `pix_on` silently degrades into `pix_off` — the comparison would then report PIX as free.
fn assert_pix_budget_remaining(mode: PixMode, generator: &SsaShareGenerator<HoprPixSpec>, pseudonym: &HoprPseudonym) {
    if matches!(mode, PixMode::On) {
        let probe = hopr_api::types::crypto_random::random_bytes::<16>();
        assert!(
            generator
                .next_share(pseudonym, &probe)
                .expect("probe must not error")
                .is_some(),
            "pix_on share budget exhausted mid-run; results are not comparable"
        );
    }
}

pub fn protocol_throughput_sender(c: &mut Criterion) {
    const PAYLOAD_SIZE: usize = HoprPacket::PAYLOAD_SIZE;
    /// Payload size `random_packets_of_count` produces, which decides how many SURBs fit alongside it.
    const BENCH_PAYLOAD_SIZE: usize = 300;
    const PEER_COUNT: usize = 3;
    const TESTED_PEER_ID: usize = 0;

    let mut group = c.benchmark_group("protocol_throughput_pipeline");
    group.sample_size(SAMPLE_SIZE);
    group.measurement_time(std::time::Duration::from_secs(30));
    for bytes in if cfg!(feature = "all-benchmarks") {
        &[5 * 1024 * 2 * PAYLOAD_SIZE, 10 * 1024 * 2 * PAYLOAD_SIZE][..]
    } else {
        &[10 * 1024 * 2 * PAYLOAD_SIZE][..]
    } {
        group.throughput(Throughput::Bytes(*bytes as u64));
        for pix in PixMode::ALL {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!(
                    "random_data_size_{}/{}",
                    bytesize::ByteSize::b(*bytes as u64).to_string().replace(" ", "_"),
                    pix.as_str()
                )),
                bytes,
                |b, bytes| {
                    let packets = random_packets_of_count(*bytes / PAYLOAD_SIZE);

                    // Built once per benchmark id: committing an SSA costs seconds, and the
                    // encoder is rebuilt every iteration, so the generator (and the pseudonym it
                    // is committed for) has to outlive the iterations. `EntryShareGenerator` is
                    // auto-implemented for `Arc`, so the encoder can take a cheap clone.
                    let (pix_gen, pix_pseudonym) = pix.build();
                    let pix_gen_probe = pix_gen.clone();

                    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime must be constructible");
                    let (node_dbs, connectors) = runtime.block_on(async {
                        let mut node_dbs = Vec::new();
                        let mut connectors = Vec::new();
                        for i in 0..PEER_COUNT {
                            let node_db = Arc::new(HoprTicketFactory::new(RedbStore::new_temp().unwrap()));
                            node_dbs.push(node_db);

                            let mut connector = create_trustful_hopr_blokli_connector(
                                &PEERS_CHAIN[i],
                                Default::default(),
                                CHAIN_DATA.clone().build_static_client(),
                                Default::default(),
                            )
                            .await
                            .expect("connector must be constructible");

                            connector.connect().await.expect("connector must be connected");
                            connectors.push(Arc::new(connector));
                        }
                        (node_dbs, connectors)
                    });

                    b.to_async(runtime).iter(|| {
                        let packets = packets.clone();
                        let node_dbs = node_dbs.clone();
                        let connectors = connectors.clone();
                        let ssa_gen = pix_gen.clone();

                        async move {
                            let (received_ack_tickets_tx, _received_ack_tickets_rx) =
                                futures::channel::mpsc::unbounded::<TicketEvent>();

                            let (wire_out_tx, wire_out_rx) = futures::channel::mpsc::unbounded::<(PeerId, Bytes)>();

                            let (_wire_in_tx, wire_in_rx) = futures::channel::mpsc::unbounded::<(PeerId, Bytes)>();

                            let (api_send_tx, api_send_rx) = futures::channel::mpsc::unbounded::<(
                                ResolvedTransportRouting<HoprSurb>,
                                ApplicationDataOut,
                            )>();
                            let (api_recv_tx, _api_recv_rx) =
                                futures::channel::mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

                            let surb_store = MemorySurbStore::new(SurbStoreConfig::default());
                            let channels_dst = connectors[TESTED_PEER_ID].domain_separators().await.unwrap().channel;

                            let codec_config = HoprCodecConfig {
                                outgoing_ticket_price: Some(HoprBalance::from_str("0.1 wxHOPR").unwrap()),
                                outgoing_win_prob: Some(WinningProbability::ALWAYS),
                                ..Default::default()
                            };

                            let ticket_proc = HoprUnacknowledgedTicketProcessor::new(
                                connectors[TESTED_PEER_ID].clone(),
                                PEERS_CHAIN[TESTED_PEER_ID].clone(),
                                channels_dst,
                                HoprUnacknowledgedTicketProcessorConfig::default(),
                            );

                            let encoder = HoprEncoder::new(
                                PEERS_CHAIN[TESTED_PEER_ID].clone(),
                                connectors[TESTED_PEER_ID].clone(),
                                surb_store.clone(),
                                node_dbs[TESTED_PEER_ID].clone(),
                                channels_dst,
                                ssa_gen,
                                codec_config,
                            );

                            let decoder = HoprDecoder::new(
                                (PEERS[TESTED_PEER_ID].clone(), PEERS_CHAIN[TESTED_PEER_ID].clone()),
                                connectors[TESTED_PEER_ID].clone(),
                                surb_store,
                                node_dbs[TESTED_PEER_ID].clone(),
                                channels_dst,
                                codec_config,
                            );

                            let processes =
                                hopr_transport::protocol::PacketPipelineBuilder::new(PEERS[TESTED_PEER_ID].clone())
                                    .transport((wire_out_tx, wire_in_rx))
                                    .codec((encoder, decoder))
                                    .api((api_recv_tx, api_send_rx))
                                    .with_ticket_processing(ticket_proc, received_ack_tickets_tx)
                                    .build_for_relay();

                            let path = resolve_mock_path(
                                PEERS_CHAIN[TESTED_PEER_ID].public().to_address(),
                                PEERS_CHAIN[1..PEER_COUNT]
                                    .iter()
                                    .map(|key| key.public().to_address())
                                    .collect(),
                            )
                            .await
                            .expect("path must be constructible");

                            // A single return path, i.e. one SURB per packet. PIX shares ride on
                            // SURBs, so without a return path the generator is never consulted and
                            // the two ids in this group would be identical. One rather than two
                            // because `random_packets_of_count` produces 300-byte payloads and
                            // `HoprPacket::max_message_with_surbs(2)` is below that.
                            //
                            // The fit is asserted rather than argued, because getting it wrong does
                            // not fail — `assert!(v.await.is_ok())` below only checks the send into
                            // `api_send_tx`, an encode failure inside the pipeline is logged and the
                            // packet dropped, and `wire_out_rx.take(count)` then waits forever for
                            // packets that will never arrive. A future change to `HoprSurb::SIZE`
                            // would hang this benchmark instead of reporting its cause.
                            assert!(
                                BENCH_PAYLOAD_SIZE <= HoprPacket::max_message_with_surbs(1),
                                "a {BENCH_PAYLOAD_SIZE}-byte payload must fit alongside one SURB"
                            );
                            // The forward path is reused as the return path. A topologically correct
                            // return path is not constructible here: `MockPathResolver` resolves
                            // against `CHANNELS`, which is a one-directional chain
                            // (peer0 -> peer1 -> peer2), so nothing leaves the last peer. What the
                            // SURB construction cost depends on is the path *length* — a share is
                            // embedded only when the return path has at least one relayer — and
                            // nothing here ever redeems a SURB, so the direction is immaterial to
                            // what is being measured.
                            let return_path = path.clone();

                            let routing = ResolvedTransportRouting::Forward {
                                // Must be the pseudonym the generator was committed for, otherwise
                                // `next_share` finds no polynomials and takes the PIX-off path.
                                pseudonym: pix_pseudonym,
                                forward_path: path,
                                return_paths: vec![return_path],
                            };

                            let count = packets.len();
                            futures::stream::iter(packets)
                                .map(|packet| {
                                    let mut sender = api_send_tx.clone();
                                    let path = routing.clone();

                                    async move {
                                        sender
                                            .send((path.clone(), ApplicationDataOut::with_no_packet_info(packet)))
                                            .await
                                    }
                                })
                                .for_each_concurrent(Some(50), |v| async {
                                    assert!(v.await.is_ok());
                                })
                                .await;

                            assert_eq!(wire_out_rx.take(count).count().await, count);

                            processes.abort_all();
                        }
                    });

                    assert_pix_budget_remaining(pix, &pix_gen_probe, &pix_pseudonym);
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, protocol_throughput_sender,);
criterion_main!(benches);
