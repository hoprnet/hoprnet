//! Integration coverage for the Start-protocol dispatch drop paths.
//!
//! These run against the compiled library rather than `cfg(test)`, which is what makes them able to
//! reach the `#[cfg(all(feature = "telemetry", not(test)))]` metric calls the unit tests cannot.

use futures::{StreamExt, channel::mpsc::UnboundedSender};
use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
use hopr_protocol_app::v1::{ApplicationData, ApplicationDataIn};
use hopr_protocol_start::KeepAliveMessage;
use hopr_transport_session::{
    ApplicationDataOut, DestinationRouting, DispatchResult, DropReason, HoprStartProtocol, MockMsgSender,
    SessionManager, testing::mock_packet_planning,
};

type TestManager = SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>;

async fn started_manager() -> anyhow::Result<TestManager> {
    let mgr: TestManager = SessionManager::new(Default::default());
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
