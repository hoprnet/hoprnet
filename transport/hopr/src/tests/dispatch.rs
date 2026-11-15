use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use hopr_api::types::crypto_random::Randomizable;
use hopr_network_graph::ChannelGraph;
use hopr_protocol_app::prelude::ReservedTag;
use hopr_protocol_start::{StartChallenge, StartInitiation};
use hopr_transport_probe::{ProbeClassifierState, content::Message, types::NeighborProbe};
use hopr_transport_session::{HoprSessionCapabilities, HoprStartProtocol};
use tracing::instrument::WithSubscriber;

use crate::{tests::support::Logs, trace_context::WithContext, *};

struct NoProbes;

impl ProbingTrafficGeneration for NoProbes {
    fn build(&self) -> futures::stream::BoxStream<'static, hopr_api::ct::ProbeRouting> {
        Box::pin(futures::stream::pending())
    }
}

async fn classifier() -> anyhow::Result<ProbeClassifierState<ChannelGraph>> {
    let allocator = hopr_transport_tag_allocator::create_allocators_from_config(&Default::default())?
        .pop()
        .context("missing probe tag allocator")?
        .1;
    let (processes, classifier) = Probe::new(Default::default(), allocator)
        .continuously_scan(
            futures::sink::drain(),
            futures::stream::pending(),
            NoProbes,
            ChannelGraph::new(*OffchainKeypair::random().public()),
        )
        .await;
    processes.abort_all();
    Ok(classifier)
}

fn incoming(data: ApplicationData) -> ApplicationDataIn {
    ApplicationDataIn {
        data,
        packet_info: Default::default(),
    }
}

/// Probes are consumed, session outcomes stay isolated, and unrelated payloads reach the caller.
#[tokio::test]
#[serial_test::serial(routing_logs)]
async fn dispatch_preserves_context_and_continues_after_drops_and_errors() -> anyhow::Result<()> {
    let logs = Logs::default();
    async {
        let probe_classifier = classifier().await?;
        let manager = Arc::new(HoprSessionManager::new(Default::default()));
        let (out_tx, out_rx) = bounded_sink_channel(8);
        let out_tx = HoprMessageSender::new(out_tx);
        let mut processes = AbortableList::default();
        for (index, handle) in manager
            .start(out_tx.clone(), futures::sink::drain(), None, None)?
            .into_iter()
            .enumerate()
        {
            processes.insert(index, handle);
        }
        let id = HoprPseudonym::random();
        let ping = Message::Probe(NeighborProbe::Ping([7; NeighborProbe::NONCE_SIZE]));
        let opening = HoprStartProtocol::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::UdpStream(hopr_utils::network_types::prelude::SealedHost::Plain(
                std::net::SocketAddr::from(([127, 0, 0, 1], 1234)).into(),
            )),
            capabilities: HoprSessionCapabilities::try_from(0u8)?,
            additional_data: 0,
        });
        let opening: ApplicationData = opening.try_into()?;
        let payload = ApplicationData::new(1234u64, b"application payload")?;
        let probe_span = tracing::debug_span!("probe.packet");
        let expected_probe_context = probe_span.id();
        let input = vec![
            WithContext::new((id, incoming(ping.try_into()?)), probe_span),
            WithContext::new((id, incoming(opening.clone())), tracing::debug_span!("opening.packet")),
            WithContext::new(
                (
                    HoprPseudonym::random(),
                    incoming(ApplicationData::new(ReservedTag::Session, b"late session data")?),
                ),
                tracing::debug_span!("late.packet"),
            ),
            WithContext::new(
                (
                    id,
                    incoming(ApplicationData::new(ReservedTag::SessionStart, vec![255])?),
                ),
                tracing::debug_span!("malformed.packet"),
            ),
            WithContext::new(
                (id, incoming(payload.clone())),
                tracing::debug_span!("application.packet"),
            ),
        ];
        let (app_tx, app_rx) = bounded_sink_channel(8);
        dispatch_incoming_messages(futures::stream::iter(input), probe_classifier, out_tx, manager, app_tx).await;
        let delivered = app_rx.collect::<Vec<_>>().await;
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].data, payload);
        futures::pin_mut!(out_rx);
        let reply = tokio::time::timeout(Duration::from_secs(2), out_rx.next())
            .await
            .context("probe reply timed out")?
            .context("probe must produce a pong")?;
        let reply_context = reply.span().id();
        assert_eq!(reply.value.0, DestinationRouting::Return(id.into()));
        assert_eq!(
            Message::try_from(reply.value.1.data)?,
            Message::Probe(NeighborProbe::Pong([7; NeighborProbe::NONCE_SIZE]))
        );
        if cfg!(feature = "tracing-context") {
            assert_eq!(reply_context, expected_probe_context);
        }
        // A valid opening before the manager starts is a real dispatch error, unlike malformed
        // network input, which is intentionally a benign drop. Later application data still flows.
        let (out_tx, _out_rx) = bounded_sink_channel(1);
        let (app_tx, app_rx) = bounded_sink_channel(1);
        let input = vec![
            WithContext::new((id, incoming(opening)), tracing::debug_span!("not_started.packet")),
            WithContext::new((id, incoming(payload.clone())), tracing::Span::none()),
        ];
        dispatch_incoming_messages(
            futures::stream::iter(input),
            classifier().await?,
            HoprMessageSender::new(out_tx),
            Arc::new(HoprSessionManager::new(Default::default())),
            app_tx,
        )
        .await;
        let delivered = app_rx.collect::<Vec<_>>().await;
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].data, payload);
        for (message, parent) in [
            ("message dispatch completed", "opening.packet"),
            (
                "error while dispatching packet in the session manager",
                "not_started.packet",
            ),
        ] {
            let records = logs.records(message);
            assert_eq!(records.len(), 1, "{message}");
            assert_eq!(records[0].ancestry, [parent, "session.dispatch"], "{}", logs.output());
        }
        let dropped = logs.records("dropped session packet");
        assert_eq!(dropped.len(), 2);
        assert_eq!(dropped[0].ancestry, ["late.packet", "session.dispatch"]);
        assert_eq!(dropped[1].ancestry, ["malformed.packet", "session.dispatch"]);
        let unrelated = logs.records("unrelated message dispatch completed");
        assert_eq!(unrelated.len(), 2);
        assert_eq!(unrelated[0].ancestry, ["application.packet", "session.dispatch"]);
        assert_eq!(unrelated[1].ancestry, ["session.dispatch"]);
        let probes = logs.records("received ping");
        assert_eq!(probes.len(), 1);
        assert_eq!(probes[0].ancestry, ["probe.packet"]);
        anyhow::Ok(())
    }
    .with_subscriber(logs.subscriber(tracing::Level::TRACE))
    .await
}

/// Dropping the application socket must not prevent later probes from being answered.
#[tokio::test]
#[serial_test::serial(routing_logs)]
async fn dispatch_keeps_serving_probes_after_the_application_receiver_closes() -> anyhow::Result<()> {
    let id = HoprPseudonym::random();
    let (out_tx, out_rx) = bounded_sink_channel(8);
    let (app_tx, app_rx) = bounded_sink_channel(1);
    drop(app_rx);
    let ping = Message::Probe(NeighborProbe::random_nonce());
    let input = [ApplicationData::new(1234u64, b"unread")?, ping.try_into()?]
        .into_iter()
        .map(|data| WithContext::new((id, incoming(data)), tracing::Span::none()));
    dispatch_incoming_messages(
        futures::stream::iter(input),
        classifier().await?,
        HoprMessageSender::new(out_tx),
        Arc::new(HoprSessionManager::new(Default::default())),
        app_tx,
    )
    .await;
    let replies = out_rx.collect::<Vec<_>>().await;
    assert_eq!(
        replies.len(),
        1,
        "closed application socket must not stop probe handling"
    );
    assert!(matches!(
        Message::try_from(replies[0].value.1.data.clone())?,
        Message::Probe(NeighborProbe::Pong(_))
    ));
    Ok(())
}

/// The real dispatch loop yields even when every packet is consumed without an application send.
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(routing_logs)]
async fn sessions_management_dispatch_loop_yields_to_scheduler() -> anyhow::Result<()> {
    let classifier = classifier().await?;
    let counter = Arc::new(AtomicU64::new(0));
    let counter_clone = counter.clone();
    let canary = hopr_utils::runtime::prelude::spawn(async move {
        loop {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            hopr_utils::runtime::prelude::yield_now().await;
        }
    });
    let packet = incoming(ApplicationData::new(ReservedTag::Session, b"late data")?);
    let id = HoprPseudonym::random();
    let input = (0..1000).map(|_| WithContext::new((id, packet.clone()), tracing::Span::none()));
    let (out_tx, _out_rx) = bounded_sink_channel(1);
    let (app_tx, app_rx) = bounded_sink_channel(1);
    dispatch_incoming_messages(
        futures::stream::iter(input),
        classifier,
        HoprMessageSender::new(out_tx),
        Arc::new(HoprSessionManager::new(Default::default())),
        app_tx,
    )
    .await;
    let progress = counter.load(Ordering::Relaxed);
    canary.abort();
    let _ = canary.await;
    assert!(progress > 0, "dispatch must allow another task to make progress");
    assert_eq!(app_rx.count().await, 0, "late session data must be dropped");
    Ok(())
}
