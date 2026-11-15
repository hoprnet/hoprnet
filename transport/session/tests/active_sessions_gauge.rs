//! Integration coverage for the active-sessions gauge, against the compiled library so that the
//! `cfg(not(test))` metric calls are reached.
//!
//! A binary of its own, because the gauge is process-global: a Session of another test would move it.

use std::time::Duration;

use anyhow::Context;
use futures::channel::mpsc::UnboundedSender;
use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
use hopr_protocol_app::v1::{ApplicationData, ApplicationDataIn};
use hopr_protocol_start::StartInitiation;
use hopr_transport_session::{
    ApplicationDataOut, DestinationRouting, HoprSessionCapabilities, HoprStartProtocol, SessionManager, SessionTarget,
};
use hopr_utils::network_types::prelude::SealedHost;

type TestManager = SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>;

/// A Session closed while the Exit is still establishing it, whose establishment then fails, leaves
/// the gauge at `num_active_sessions`.
///
/// The Exit publishes the slot before it notifies about the new Session, so the close finds it there;
/// the notification then times out, which fails the establishment after the close.
#[test_log::test(tokio::test)]
async fn a_session_closed_before_its_establishment_fails_leaves_the_gauge_balanced() -> anyhow::Result<()> {
    let mgr: TestManager = SessionManager::new(Default::default());
    let (msg_sender, _sent) = futures::channel::mpsc::unbounded();
    // No buffer and never read: the first notification parks the sender until its send times out.
    let (new_session_tx, _new_sessions) = futures::channel::mpsc::channel(0);
    let _handles = mgr.start(msg_sender, new_session_tx, None, None)?;

    let pseudonym = HoprPseudonym::random();
    let offer = StartInitiation {
        challenge: 1,
        target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().context("target must parse")?)),
        capabilities: HoprSessionCapabilities::empty(),
        additional_data: 0,
    };
    mgr.dispatch_message(
        pseudonym,
        ApplicationDataIn {
            data: ApplicationData::try_from(HoprStartProtocol::StartSession(offer))?,
            packet_info: Default::default(),
        },
    )?;

    tokio::time::timeout(Duration::from_secs(5), async {
        while mgr.num_active_sessions() == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .context("the slot was never allocated")?;
    assert!(
        mgr.close_session(&pseudonym),
        "the slot must be found while the Session is set up"
    );

    // Outlast the notification's send timeout, so the establishment has failed.
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(mgr.num_active_sessions(), 0);

    #[cfg(feature = "telemetry")]
    {
        let text = hopr_api::types::telemetry::gather_all_metrics().context("must gather metrics")?;
        let gauge: f64 = text
            .lines()
            .find_map(|line| line.strip_prefix("hopr_session_num_active_sessions "))
            .context("the gauge must be exported")?
            .trim()
            .parse()?;
        assert_eq!(gauge, 0.0, "the gauge must match num_active_sessions:\n{text}");
    }
    Ok(())
}
