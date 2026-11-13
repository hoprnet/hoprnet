//! Integration coverage for the Start-protocol dispatch drop paths.
//!
//! These run against the compiled library rather than `cfg(test)`, which is what makes them able to
//! reach the `#[cfg(all(feature = "telemetry", not(test)))]` metric calls the unit tests cannot.

use futures::{StreamExt, channel::mpsc::UnboundedSender};
use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
use hopr_protocol_app::v1::{ApplicationData, ApplicationDataIn};
use hopr_protocol_start::{KeepAliveMessage, StartInitiation};
use hopr_transport_session::{
    ApplicationDataOut, DestinationRouting, DispatchResult, DropReason, HoprSessionCapabilities, HoprStartProtocol,
    MockMsgSender, SessionManager, SessionManagerConfig, SessionTarget, testing::mock_packet_planning,
};
use hopr_utils::network_types::prelude::SealedHost;

type TestManager = SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>;

async fn started_manager() -> anyhow::Result<TestManager> {
    started_manager_with(SessionManagerConfig::default()).await
}

async fn started_manager_with(cfg: SessionManagerConfig) -> anyhow::Result<TestManager> {
    let mgr: TestManager = SessionManager::new(cfg);
    let (sender, _handle) = mock_packet_planning(MockMsgSender::new());
    let (new_session_tx, mut new_session_rx) = futures::channel::mpsc::channel(1);
    let _notifications = tokio::spawn(async move { while new_session_rx.next().await.is_some() {} });
    mgr.start(sender, new_session_tx, None, None)?;
    Ok(mgr)
}

/// A Start-tagged packet whose payload does not decode is a quiet, counted drop — including on the
/// telemetry-enabled build the integration flag compiles.
#[test_log::test(tokio::test)]
async fn a_malformed_start_message_is_a_quiet_drop() -> anyhow::Result<()> {
    let mgr = started_manager().await?;

    let in_data = ApplicationDataIn {
        data: ApplicationData::new(
            HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG,
            b"not a start message".as_slice(),
        )?,
        packet_info: Default::default(),
    };

    let outcome = mgr.dispatch_message(HoprPseudonym::random(), in_data)?;
    assert!(
        matches!(outcome, DispatchResult::Dropped(DropReason::MalformedStartMessage)),
        "a malformed Start message must be a quiet drop, got {outcome:?}"
    );

    Ok(())
}

/// A Start message for a pseudonym this node never issued is dropped before it can occupy a handler
/// — the same quiet path, exercised with telemetry compiled in.
#[test_log::test(tokio::test)]
async fn a_stale_start_message_is_a_quiet_drop() -> anyhow::Result<()> {
    let mgr = started_manager().await?;

    let stranger = HoprPseudonym::random();
    let keep_alive = HoprStartProtocol::KeepAlive(KeepAliveMessage {
        session_id: stranger,
        flags: Default::default(),
        additional_data: 0,
    });
    let (tag, bytes) = keep_alive.encode()?;
    let in_data = ApplicationDataIn {
        data: ApplicationData::new(tag, bytes.as_ref())?,
        packet_info: Default::default(),
    };

    let outcome = mgr.dispatch_message(stranger, in_data)?;
    assert!(
        matches!(outcome, DispatchResult::Dropped(DropReason::StaleStartMessage)),
        "a Start message naming no local state must be dropped quietly, got {outcome:?}"
    );

    Ok(())
}

/// Filling the Start queue past its capacity produces a quiet counted drop, on a build where the
/// queue-full metric is compiled in.
#[test_log::test(tokio::test)]
async fn a_full_start_queue_is_a_quiet_drop() -> anyhow::Result<()> {
    let mgr = started_manager_with(SessionManagerConfig {
        maximum_sessions: 1,
        ..Default::default()
    })
    .await?;

    // `StartSession` is admitted unconditionally, so it is what fills the queue. No `.await` in the
    // loop, so the processor is never polled and the queue cannot drain.
    let mut saw_full = false;
    for i in 0..2_000u64 {
        let outcome = mgr.dispatch_message(
            HoprPseudonym::random(),
            ApplicationDataIn {
                data: ApplicationData::try_from(HoprStartProtocol::StartSession(StartInitiation {
                    challenge: 1 + i,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                }))?,
                packet_info: Default::default(),
            },
        )?;
        if matches!(outcome, DispatchResult::Dropped(DropReason::StartQueueFull)) {
            saw_full = true;
            break;
        }
    }

    assert!(saw_full, "the queue must fill within 2000 StartSessions");
    Ok(())
}
