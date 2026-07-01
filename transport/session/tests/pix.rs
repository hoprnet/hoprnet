//! Integration tests for `SessionManager` PIX protocol support.

mod common;

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use futures::{AsyncWriteExt, StreamExt, pin_mut};
use hopr_api::types::{
    crypto::{keypairs::ChainKeypair, prelude::Keypair},
    crypto_random::Randomizable,
    internal::{prelude::HoprPseudonym, routing::SurbMatcher},
    primitive::prelude::Address,
};
use hopr_protocol_app::v1::ApplicationData;
use hopr_protocol_pix::{
    SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
};
use hopr_protocol_start::StartProtocolDiscriminants;
use hopr_transport_session::{
    ApplicationDataIn, Capability, DestinationRouting, HoprSessionInPixEvent, HoprSessionOutPixEvent,
    HoprStartProtocol, IncomingSessionPixConfig, PixToolbox, SessionClientConfig, SessionManager, SessionManagerConfig,
    SessionTarget, SurbBalancerConfig,
};
use hopr_utils::network_types::prelude::SealedHost;
use test_log::test;
use tokio::time as tokio_time;

use crate::common::{MockMsgSender, mock_packet_planning, msg_type};

#[test(tokio::test)]
async fn session_manager_should_follow_start_protocol_to_establish_new_session_and_close_it_with_pix() -> Result<()> {
    let alice_pseudonym = Arc::new(HoprPseudonym::random());
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let alice_mgr = SessionManager::new(Default::default());
    let bob_mgr = SessionManager::new(SessionManagerConfig {
        pix_config: IncomingSessionPixConfig {
            quota_range: 0..=2048 * 1024 * 1024,
            ..Default::default()
        },
        ..Default::default()
    });

    let ssa_gen_config = SsaGeneratorConfig {
        polynomials_per_ssa: 64,
        threshold: 64,
        surplus_shares: 16,
    };

    let expected_ssa_commits = {
        let max_commitments_per_message = (ApplicationData::PAYLOAD_SIZE - HoprStartProtocol::START_HEADER_SIZE)
            / (size_of::<SsaIndex>() + size_of::<hopr_crypto_packet::prelude::HoprPixGroupElement>());
        ssa_gen_config.threshold * ssa_gen_config.polynomials_per_ssa.div_ceil(max_commitments_per_message)
    };

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let bob_mgr_clone = Arc::new(bob_mgr.clone());
    let alice_pseudonym_for_alice_start = alice_pseudonym.clone();
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("alice sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr_clone = bob_mgr_clone.clone();
            let pseudonym = alice_pseudonym_for_alice_start.clone();
            Box::pin(async move {
                bob_mgr_clone.dispatch_message(
                    *pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Bob sends the SessionEstablished message
    let alice_mgr_session_established = Arc::new(alice_mgr.clone());
    let alice_pseudonym_est = alice_pseudonym.clone(); // for .withf()
    let alice_pseudonym_ret_est = alice_pseudonym.clone(); // for .returning()
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("bob sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == alice_pseudonym_est.as_ref())
        })
        .returning(move |_, data| {
            let mgr = alice_mgr_session_established.clone();
            let pseudonym = alice_pseudonym_ret_est.clone();
            Box::pin(async move {
                mgr.dispatch_message(
                    *pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Bob also sends SsaRequest message
    let alice_mgr_ssa_request = Arc::new(alice_mgr.clone());
    let alice_pseudonym_ssa = alice_pseudonym.clone(); // for .withf()
    let alice_pseudonym_ret_ssa = alice_pseudonym.clone(); // for .returning()
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("bob sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SsaRequest)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == alice_pseudonym_ssa.as_ref())
        })
        .returning(move |_, data| {
            let mgr = alice_mgr_ssa_request.clone();
            let pseudonym = alice_pseudonym_ret_ssa.clone();
            Box::pin(async move {
                mgr.dispatch_message(
                    *pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends the SsaCommit message
    let bob_mgr_ssa_commit = Arc::new(bob_mgr.clone());
    let alice_pseudonym_for_alice_ssa = alice_pseudonym.clone();
    alice_transport
        .expect_send_message()
        .times(expected_ssa_commits)
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("alice sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SsaCommit)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr_ssa_commit = bob_mgr_ssa_commit.clone();
            let pseudonym = alice_pseudonym_for_alice_ssa.clone();
            Box::pin(async move {
                bob_mgr_ssa_commit.dispatch_message(
                    *pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends the terminating segment to close the Session
    let bob_mgr_seg = Arc::new(bob_mgr.clone());
    let alice_pseudonym_for_alice_seg = alice_pseudonym.clone();
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            hopr_protocol_session::types::SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::try_from(
                data.data.plain_text.as_ref(),
            )
            .expect("must be a session message")
            .try_as_segment()
            .expect("must be a segment")
            .is_terminating()
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr_seg = bob_mgr_seg.clone();
            let pseudonym = alice_pseudonym_for_alice_seg.clone();
            Box::pin(async move {
                bob_mgr_seg.dispatch_message(
                    *pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    let mut ahs = Vec::new();

    let ssa_rec_config = SsaReconstructorConfig::default();

    let (pix_toolbox_alice, pix_alice_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );
    let (pix_toolbox_bob, pix_bob_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );

    // Start Alice
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, Some(pix_toolbox_alice))?);
    assert!(alice_mgr.is_started());

    // Start Bob
    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, Some(pix_toolbox_bob))?);
    assert!(bob_mgr.is_started());

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let (alice_session, bob_session) = tokio::time::timeout(
        Duration::from_secs(2),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target.clone()),
                SessionClientConfig {
                    pseudonym: (*alice_pseudonym).into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation | Capability::UsePIX,
                    surb_management: None,
                    pix_ssa_quota: Some((
                        ssa_gen_config.polynomials_per_ssa as u32,
                        ssa_gen_config.threshold as u32,
                    )),
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;

    let mut alice_session = alice_session?;
    let bob_session = bob_session.ok_or(anyhow::anyhow!("bob must get an incoming session"))?;

    assert_eq!(
        alice_session.config().capabilities,
        Capability::Segmentation | Capability::NoRateControl | Capability::UsePIX
    );
    assert_eq!(
        alice_session.config().capabilities,
        bob_session.session.config().capabilities
    );
    assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

    assert_eq!(vec![*alice_session.id()], alice_mgr.active_sessions());
    assert_eq!(None, alice_mgr.get_surb_balancer_config(alice_session.id())?);
    assert!(
        alice_mgr
            .update_surb_balancer_config(alice_session.id(), SurbBalancerConfig::default())
            .is_err()
    );

    assert_eq!(vec![*bob_session.session.id()], bob_mgr.active_sessions());
    assert_eq!(None, bob_mgr.get_surb_balancer_config(bob_session.session.id())?);
    assert!(
        bob_mgr
            .update_surb_balancer_config(bob_session.session.id(), SurbBalancerConfig::default())
            .is_err()
    );

    pin_mut!(pix_alice_rx);
    pin_mut!(pix_bob_rx);

    let alice_session_event = tokio_time::timeout(Duration::from_secs(2), pix_alice_rx.next())
        .await
        .map_err(|e| anyhow::anyhow!("timeout: {e}"))?
        .ok_or(anyhow::anyhow!("alice must get a pix event"))?;

    assert!(matches!(alice_session_event, HoprSessionOutPixEvent::ReadyToDeposit(_)));

    let bob_session_event = tokio_time::timeout(Duration::from_secs(2), pix_bob_rx.next())
        .await
        .map_err(|e| anyhow::anyhow!("timeout: {e}"))?
        .ok_or(anyhow::anyhow!("bob must get a pix event"))?;

    assert!(matches!(bob_session_event, HoprSessionOutPixEvent::DepositNeeded(..)));

    tokio::time::sleep(Duration::from_millis(100)).await;
    alice_session.close().await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(matches!(
        alice_mgr.ping_session(alice_session.id()).await,
        Err(hopr_transport_session::errors::TransportSessionError::Manager(
            hopr_transport_session::errors::SessionManagerError::NonExistingSession
        ))
    ));

    for ah in ahs {
        ah.abort();
    }

    // Cleanup: close senders and await handles
    alice_sender.close_channel();
    bob_sender.close_channel();
    let _ = alice_handle.await;
    let _ = bob_handle.await;

    Ok(())
}

#[test(tokio::test)]
async fn dispatch_pix_event_returns_error_for_unknown_session() -> Result<()> {
    let mgr = SessionManager::new(Default::default());

    let transport = MockMsgSender::new();
    let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
    let _notifications = tokio::spawn(async move {
        pin_mut!(new_session_rx);
        while let Some(_session) = new_session_rx.next().await {}
    });
    let (sender, handle) = mock_packet_planning(transport);
    mgr.start(sender.clone(), new_session_tx, None)?;
    assert!(mgr.is_started());

    let unknown_pseudonym = HoprPseudonym::random();
    let ssa_id = SsaId::new(unknown_pseudonym, SsaIndex::new(1).expect("ssa index must be non-zero"));
    let event = HoprSessionInPixEvent::UnverifiableShare(ssa_id);

    let result = mgr.dispatch_pix_event(event).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        hopr_transport_session::errors::TransportSessionError::Manager(
            hopr_transport_session::errors::SessionManagerError::NonExistingSession
        )
    ));

    sender.close_channel();
    let _ = handle.await;

    Ok(())
}
