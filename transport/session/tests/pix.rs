//! Integration tests for `SessionManager` PIX protocol support.

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
    HoprStartProtocol, IncomingSessionPixConfig, MockMsgSender, PixToolbox, SessionClientConfig, SessionManager,
    SessionManagerConfig, SessionTarget, SurbBalancerConfig,
    test_helpers::{mock_packet_planning, msg_type},
};
use hopr_utils::network_types::prelude::SealedHost;
use test_log::test;
use tokio::time as tokio_time;

/// Verifies the complete session establishment and teardown when both peers use the PIX protocol.
///
/// Unlike the vanilla lifecycle test, Bob is configured with a generous PIX quota and both peers
/// are given a `PixToolbox` so that the SSA (Secret Sharing Agreement) handshake runs as part of
/// session establishment.
///
/// ## Steps
/// 1. Alice's manager has no PIX config (initiator, no quota enforcement). Bob's manager accepts quotas up to 2 GiB via
///    `IncomingSessionPixConfig`.
/// 2. Both managers receive a `PixToolbox` seeded with a `SsaShareGenerator` and `SsaReconstructor`.
/// 3. Alice calls `new_session` with `Capability::UsePIX` and a quota of `(64, 64)`. The mock intercepts the outbound
///    messages in sequence:
///    - `StartSession` → delivered to Bob
///    - `SessionEstablished` → delivered to Alice
///    - `SsaRequest` (from Bob) → delivered to Alice
///    - `SsaCommit` messages (from Alice, one per polynomial group) → each delivered to Bob
///    - terminating segment (from Alice) → delivered to Bob
/// 4. Both sessions are established and `UsePIX | Segmentation | NoRateControl` capabilities are confirmed on both
///    sides.
/// 5. Alice receives a `HoprSessionOutPixEvent::ReadyToDeposit` on her PIX event stream, and Bob receives
///    `DepositNeeded`, confirming the SSA handshake produced events on both sides.
/// 6. Alice closes the session; `ping_session` on the closed session returns `NonExistingSession`.
#[test(tokio::test)]
async fn session_manager_should_follow_start_protocol_to_establish_new_session_and_close_it_with_pix() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
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
        ssa_gen_config.threshold as usize
            * (ssa_gen_config.polynomials_per_ssa as usize).div_ceil(max_commitments_per_message)
    };

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let bob_mgr_clone = Arc::new(bob_mgr.clone());
    let alice_pseudonym_for_alice_start = alice_pseudonym;
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
            Box::pin(async move {
                bob_mgr_clone.dispatch_message(
                    alice_pseudonym_for_alice_start,
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
    let alice_pseudonym_est = alice_pseudonym; // for .withf()
    let alice_pseudonym_ret_est = alice_pseudonym; // for .returning()
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("bob sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym_est)
        })
        .returning(move |_, data| {
            let mgr = alice_mgr_session_established.clone();
            Box::pin(async move {
                mgr.dispatch_message(
                    alice_pseudonym_ret_est,
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
    let alice_pseudonym_ssa = alice_pseudonym; // for .withf()
    let alice_pseudonym_ret_ssa = alice_pseudonym; // for .returning()
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::trace!("bob sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SsaRequest)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym_ssa)
        })
        .returning(move |_, data| {
            let mgr = alice_mgr_ssa_request.clone();
            Box::pin(async move {
                mgr.dispatch_message(
                    alice_pseudonym_ret_ssa,
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
    let alice_pseudonym_for_alice_ssa = alice_pseudonym;
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
            Box::pin(async move {
                bob_mgr_ssa_commit.dispatch_message(
                    alice_pseudonym_for_alice_ssa,
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
    let alice_pseudonym_for_alice_seg = alice_pseudonym;
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
            Box::pin(async move {
                bob_mgr_seg.dispatch_message(
                    alice_pseudonym_for_alice_seg,
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
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation | Capability::UsePIX,
                    surb_management: None,
                    pix_ssa_quota: Some((ssa_gen_config.polynomials_per_ssa, ssa_gen_config.threshold)),
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

    let HoprSessionOutPixEvent::ReadyToDeposit(alice_quota) = &alice_session_event else {
        panic!("expected ReadyToDeposit, got {alice_session_event:?}");
    };

    let bob_session_event = tokio_time::timeout(Duration::from_secs(2), pix_bob_rx.next())
        .await
        .map_err(|e| anyhow::anyhow!("timeout: {e}"))?
        .ok_or(anyhow::anyhow!("bob must get a pix event"))?;

    let HoprSessionOutPixEvent::DepositNeeded(bob_quota, _) = &bob_session_event else {
        panic!("expected DepositNeeded, got {bob_session_event:?}");
    };

    // Both peers must agree on the same SSA parameters
    assert_eq!(
        alice_quota.ssa_id, bob_quota.ssa_id,
        "Entry and Exit must agree on SSA ID"
    );
    assert_eq!(
        alice_quota.quota_per_ssa, bob_quota.quota_per_ssa,
        "Entry and Exit must agree on SSA quota"
    );

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

/// Verifies that dispatching a PIX event to a session that does not exist returns a
/// `NonExistingSession` error.
///
/// ## Steps
/// 1. A `SessionManager` is started without a `PixToolbox`.
/// 2. An `UnverifiableShare` event is constructed with a random (unknown) `SsaId`.
/// 3. `dispatch_pix_event` is called on the manager with this unknown session ID.
/// 4. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::NonExistingSession)`,
///    confirming the manager correctly rejects PIX events for sessions it does not hold.
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

/// Verifies that when neither side has a `PixToolbox`, the session establishes without any SSA
/// exchange — the PIX protocol is simply skipped.
///
/// ## Steps
/// 1. Both managers are started without a `PixToolbox`.
/// 2. Alice initiates a session without `Capability::UsePIX` — neither side can run the PIX state machine.
/// 3. The mock captures and delivers `StartSession` → Bob and `SessionEstablished` → Alice.
/// 4. No `SsaRequest` is sent by Bob (Bob has no toolbox to generate one).
/// 5. Both sessions are established and both sides receive a session handle.
#[test(tokio::test)]
async fn exit_without_pix_toolbox_does_not_send_ssa_request() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let alice_mgr = SessionManager::new(Default::default());
    let bob_mgr = SessionManager::new(SessionManagerConfig {
        pix_config: IncomingSessionPixConfig {
            quota_range: 0..=2048 * 1024 * 1024,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    let bob_mgr_clone = Arc::new(bob_mgr.clone());
    let alice_pseudonym_for_alice_start = alice_pseudonym;
    alice_transport.expect_send_message().returning(move |_, data| {
        let bob_mgr_clone = bob_mgr_clone.clone();
        Box::pin(async move {
            bob_mgr_clone.dispatch_message(
                alice_pseudonym_for_alice_start,
                ApplicationDataIn {
                    data: data.data,
                    packet_info: Default::default(),
                },
            )?;
            Ok(())
        })
    });

    let alice_mgr_session_established = Arc::new(alice_mgr.clone());
    let alice_pseudonym_ret_est = alice_pseudonym;
    bob_transport.expect_send_message().times(1).returning(move |_, data| {
        let mgr = alice_mgr_session_established.clone();
        Box::pin(async move {
            mgr.dispatch_message(
                alice_pseudonym_ret_est,
                ApplicationDataIn {
                    data: data.data,
                    packet_info: Default::default(),
                },
            )?;
            Ok(())
        })
    });

    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);

    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1);
    alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;

    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1);
    bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?;

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let (alice_session, bob_session_option) = tokio::time::timeout(
        Duration::from_secs(2),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::Segmentation.into(),
                    surb_management: None,
                    pix_ssa_quota: None,
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;
    let _alice_session = alice_session?;
    let _bob_session = bob_session_option.ok_or(anyhow::anyhow!("bob must get an incoming session"))?;

    alice_sender.close_channel();
    bob_sender.close_channel();
    let _ = alice_handle.await;
    let _ = bob_handle.await;

    Ok(())
}
