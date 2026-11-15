//! Integration coverage for the refusal reply budget, against the compiled library so that the
//! `cfg(not(test))` metric calls are reached.

use anyhow::Context;
use futures::channel::mpsc::UnboundedSender;
use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
use hopr_protocol_app::v1::{ApplicationData, ApplicationDataIn};
use hopr_protocol_start::{StartInitiation, StartProtocolDiscriminants};
use hopr_transport_session::{
    ApplicationDataOut, DestinationRouting, DispatchResult, HoprSessionCapabilities, HoprStartProtocol,
    RefusalReplyBudget, SessionManager, SessionManagerConfig, SessionTarget, counters::start_refusal_reply_drop_count,
    msg_type,
};
use hopr_utils::network_types::prelude::SealedHost;

type TestManager = SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>;

fn opening() -> anyhow::Result<ApplicationDataIn> {
    let offer = StartInitiation {
        challenge: 0x1000,
        target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().context("target must parse")?)),
        capabilities: HoprSessionCapabilities::empty(),
        additional_data: 0,
    };
    Ok(ApplicationDataIn {
        data: ApplicationData::try_from(HoprStartProtocol::StartSession(offer))?,
        packet_info: Default::default(),
    })
}

/// Refusals beyond the budget get no reply, and the replies sent are counted as errors sent.
#[test_log::test(tokio::test)]
async fn refusals_beyond_the_budget_are_dropped_without_a_reply() -> anyhow::Result<()> {
    let mgr: TestManager = SessionManager::new(SessionManagerConfig {
        maximum_sessions: 0,
        refusal_reply_budget: Some(RefusalReplyBudget {
            burst: 2,
            refill_period: std::time::Duration::from_secs(3600),
        }),
        ..Default::default()
    });
    let (mock_sender, mut sent) = futures::channel::mpsc::unbounded();
    let (new_session_tx, _new_sessions) = futures::channel::mpsc::channel(1);
    let _handles = mgr.start(mock_sender, new_session_tx, None, None)?;
    let drops_before = start_refusal_reply_drop_count();

    for _ in 0..5 {
        let result = mgr.dispatch_message(HoprPseudonym::random(), opening()?)?;
        assert!(matches!(result, DispatchResult::Processed), "dispatched as {result:?}");
    }

    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while start_refusal_reply_drop_count() < drops_before + 3 {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .context("the surplus refusals were never dropped")?;

    let mut replies = 0;
    while let Ok((_, data)) = sent.try_recv() {
        assert!(msg_type(&data, StartProtocolDiscriminants::SessionError));
        replies += 1;
    }
    assert_eq!(2, replies, "exactly the burst is answered");

    #[cfg(feature = "telemetry")]
    {
        let text = hopr_api::types::telemetry::gather_all_metrics().context("must gather metrics")?;
        assert!(
            text.lines()
                .any(|line| line.starts_with("hopr_exit_control_packets_total{")
                    && line.contains("outcome=\"dropped_budget\"")
                    && line.contains("kind=\"refusal_no_session_slot\"")),
            "dropped refusals were not counted:\n{text}"
        );
        assert!(
            text.lines()
                .any(|line| line.starts_with("hopr_session_sent_error_count")),
            "sent refusals were not counted as errors sent:\n{text}"
        );
    }
    Ok(())
}
