//! Integration tests for `SessionManager` PIX protocol support.

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use futures::{AsyncWriteExt, StreamExt, pin_mut};
use hopr_api::{
    node::PixAddressId,
    types::{
        crypto::{keypairs::ChainKeypair, prelude::Keypair},
        crypto_random::Randomizable,
        internal::{
            prelude::HoprPseudonym,
            routing::{RoutingOptions, SurbMatcher},
        },
        primitive::prelude::Address,
    },
};
use hopr_crypto_packet::HoprPixSpec;
use hopr_protocol_app::v1::ApplicationData;
use hopr_protocol_pix::{
    SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
};
use hopr_protocol_start::StartProtocolDiscriminants;
use hopr_transport_session::{
    ApplicationDataIn, Capability, DestinationRouting, HoprSessionInPixEvent, HoprSessionOutPixEvent,
    HoprStartProtocol, IncomingSessionPixConfig, MockMsgSender, PixParams, PixToolbox, SessionClientConfig,
    SessionManager, SessionManagerConfig, SessionTarget, SurbBalancerConfig,
    testing::{answering_deposit_pool, mock_packet_planning, msg_type},
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

    // One commitment per polynomial — the constant term — chunked into packet-sized messages.
    // Every message carries the proof of knowledge, so the per-message budget loses its size.
    //
    // Asked of the encoder rather than restated here: `SessionId` is an alias of `HoprPseudonym`
    // (fixed size, so its CBOR length does not depend on which pseudonym), which makes
    // `alice_pseudonym` a faithful stand-in for the session id the encoder will see.
    let expected_ssa_commits = (ssa_gen_config.polynomials_per_ssa as usize)
        .div_ceil(HoprStartProtocol::ssa_commit_chunking(&alice_pseudonym)?.max_constant_terms_per_message);

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
    // Bob is the Exit, and blocks his SSA request on the deposit pool — stand in for one so that
    // establishment does not wait out `DEPOSIT_DATA_REQUEST_TIMEOUT`.
    let pix_bob_rx = answering_deposit_pool(pix_bob_rx, |_| Vec::new());

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
                    pix_ssa_quota: Some(PixParams::try_from_config::<HoprPixSpec>(&ssa_gen_config)?),
                    return_path_options: RoutingOptions::Hops(1.try_into()?),
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
    alice_handle.await??;
    bob_handle.await??;

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
    handle.await??;

    Ok(())
}

/// Verifies that a session which does not ask for PIX establishes in exactly two messages, with no
/// SSA exchange anywhere in between.
///
/// ## Steps
/// 1. Both managers are started without a `PixToolbox`.
/// 2. Alice initiates with `Capability::Segmentation` only and `pix_ssa_quota: None`, so PIX is never negotiated.
/// 3. The mock captures and delivers `StartSession` → Bob and `SessionEstablished` → Alice, and the `.times(1)` on
///    *both* transports is what pins the absence of a third message.
/// 4. Both sessions are established and both sides receive a session handle.
///
/// Note what this does *not* show: the absent `PixToolbox` is not the operative cause here, because
/// no `SsaRequest` would be sent for an un-negotiated session in any case. Refusal on a missing
/// toolbox is covered by `incoming_usepix_session_is_rejected_when_no_pix_toolbox_is_installed` in
/// `transport/session/src/manager.rs`.
#[test(tokio::test)]
async fn session_without_pix_establishes_without_an_ssa_exchange() -> Result<()> {
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
    // `.times(1)`: Alice must send `StartSession` and nothing else. Bob's own `.times(1)` bounds
    // this only indirectly — an `SsaCommit` can just about be argued impossible because it must
    // follow an `SsaRequest` from Bob — and an unexpected `SsaCommit` is the regression this test
    // exists to catch, so the bound belongs on the side that would emit it.
    alice_transport
        .expect_send_message()
        .times(1)
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
    alice_handle.await??;
    bob_handle.await??;

    Ok(())
}

/// End-to-end check of a batched SSA exchange: one `SsaRequest` carrying several commitments must
/// produce that many independent deposit cycles on both sides.
///
/// This is the whole point of the batching knobs, and it is only observable across the full exchange:
/// the Exit packs `ssas_per_request` commitments into a single message, the Entry loops over them
/// generating one client commitment and one deposit address each, and both sides then emit one event
/// per cycle.
///
/// ## Steps
/// 1. Bob (Exit) is configured with `ssas_per_request: 3`; Alice (Entry) with a matching `max_ssas_per_ssa_request: 3`,
///    without which the request would be rejected wholesale.
/// 2. Both transports relay every Start protocol message to the peer manager, counting how many of them are
///    `SsaRequest`s.
/// 3. Exactly one `SsaRequest` goes out — the batch is one message, not three.
/// 4. Alice emits 3 `ReadyToDeposit` and Bob 3 `DepositNeeded`, at contiguous SSA indices 1..=3, with pairwise distinct
///    deposit addresses. Distinctness is what shows the batch produced genuinely separate cycles rather than the same
///    cycle reported repeatedly.
#[test(tokio::test)]
async fn batched_ssa_request_produces_one_deposit_cycle_per_requested_ssa() -> Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    const BATCH: usize = 3;

    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    // Small dimensions keep the commitment exchange to a handful of messages.
    let ssa_gen_config = SsaGeneratorConfig {
        polynomials_per_ssa: 8,
        threshold: 2,
        surplus_shares: 2,
    };

    let alice_mgr = SessionManager::new(SessionManagerConfig {
        max_ssas_per_ssa_request: BATCH,
        ..Default::default()
    });
    let bob_mgr = SessionManager::new(SessionManagerConfig {
        pix_config: IncomingSessionPixConfig {
            quota_range: 0..=2048 * 1024 * 1024,
            ssas_per_request: BATCH,
            ..Default::default()
        },
        ..Default::default()
    });

    // Plain relays in both directions — the message *ordering* is not what this test is about, so no
    // mockall sequence; only the number of SsaRequests is pinned.
    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // `.times(1..)` rather than the default of exactly one: a batch of 3 puts an unbounded number of
    // SsaCommit messages on the wire and the count is not what this test pins.
    let bob_mgr_relay = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .times(1..)
        .returning(move |_, data| {
            let bob_mgr_relay = bob_mgr_relay.clone();
            Box::pin(async move {
                // Session data segments are unrelated here and are simply dropped by the dispatcher.
                let _ = bob_mgr_relay.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                );
                Ok(())
            })
        });

    let ssa_requests = Arc::new(AtomicUsize::new(0));
    let ssa_requests_relay = ssa_requests.clone();
    let alice_mgr_relay = Arc::new(alice_mgr.clone());
    bob_transport
        .expect_send_message()
        .times(1..)
        .returning(move |_, data| {
            if msg_type(&data, StartProtocolDiscriminants::SsaRequest) {
                ssa_requests_relay.fetch_add(1, Ordering::Relaxed);
            }
            let alice_mgr_relay = alice_mgr_relay.clone();
            Box::pin(async move {
                let _ = alice_mgr_relay.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                );
                Ok(())
            })
        });

    let ssa_rec_config = SsaReconstructorConfig::default();
    let (pix_toolbox_alice, pix_alice_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );
    let (pix_toolbox_bob, pix_bob_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );

    // Bob is the Exit, so his SSA requests block on the deposit pool: stand in for one, answering
    // each SSA with bytes derived from its own index so that what comes back on either side can only
    // match if it stayed with the SSA it was produced for.
    //
    // Installed before the managers start, and it has to be: Bob's first request goes out during
    // establishment, and `new_session` can return before it is answered because `SessionEstablished`
    // precedes the PIX setup. Attaching the pool afterwards leaves that first request unanswered for
    // however long the test takes to get here — under load, long enough to hit
    // `DEPOSIT_DATA_REQUEST_TIMEOUT` and lose the Session.
    let pix_bob_rx = answering_deposit_pool(pix_bob_rx, |id| vec![id.ssa_index().get() as u8; 4]);

    let mut ahs = Vec::new();
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, Some(pix_toolbox_alice))?);

    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, Some(pix_toolbox_bob))?);

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let (alice_session, bob_session) = tokio::time::timeout(
        Duration::from_secs(5),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target.clone()),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation | Capability::UsePIX,
                    surb_management: None,
                    pix_ssa_quota: Some(PixParams::try_from_config::<HoprPixSpec>(&ssa_gen_config)?),
                    return_path_options: RoutingOptions::Hops(1.try_into()?),
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;

    let mut alice_session = alice_session?;
    let _bob_session = bob_session.ok_or(anyhow::anyhow!("bob must get an incoming session"))?;

    pin_mut!(pix_alice_rx);
    pin_mut!(pix_bob_rx);

    // One cycle per requested SSA on the Entry side.
    let mut entry_cycles = Vec::new();
    for i in 0..BATCH {
        let event = tokio_time::timeout(Duration::from_secs(5), pix_alice_rx.next())
            .await
            .map_err(|e| anyhow::anyhow!("timeout awaiting entry cycle {i}: {e}"))?
            .ok_or(anyhow::anyhow!("entry must emit cycle {i}"))?;
        let HoprSessionOutPixEvent::ReadyToDeposit(quota) = event else {
            panic!("expected ReadyToDeposit, got {event:?}");
        };
        entry_cycles.push(quota);
    }

    // And one per requested SSA on the Exit side.
    let mut exit_cycles = Vec::new();
    for i in 0..BATCH {
        let event = tokio_time::timeout(Duration::from_secs(5), pix_bob_rx.next())
            .await
            .map_err(|e| anyhow::anyhow!("timeout awaiting exit cycle {i}: {e}"))?
            .ok_or(anyhow::anyhow!("exit must emit cycle {i}"))?;
        let HoprSessionOutPixEvent::DepositNeeded(quota, _) = event else {
            panic!("expected DepositNeeded, got {event:?}");
        };
        exit_cycles.push(quota);
    }

    // The Exit notices the batch in its own order, and nothing says it should be the Entry's. The
    // Entry walks the request's `BTreeMap` of commitments in one sequential pass, so it emits
    // ascending; the Exit's `DepositNeeded` follows its `SsaCommit`s, which the manager processes
    // with `for_each_concurrent` and which therefore complete in whatever order they finish. Sorted
    // so that everything below pairs the two sides by SSA rather than by arrival — which is what all
    // of it means, and what the relays above already say by not pinning message order.
    exit_cycles.sort_by_key(|q| q.ssa_id.ssa_index());

    assert_eq!(
        1,
        ssa_requests.load(Ordering::Relaxed),
        "the whole batch must travel in a single SsaRequest"
    );

    // Contiguous indices starting at 1, and the two sides agree on every cycle.
    let entry_indices: Vec<_> = entry_cycles.iter().map(|q| q.ssa_id.ssa_index().get()).collect();
    let exit_indices: Vec<_> = exit_cycles.iter().map(|q| q.ssa_id.ssa_index().get()).collect();
    assert_eq!(
        entry_indices,
        (1..=BATCH as u32).collect::<Vec<_>>(),
        "the batch must cover contiguous SSA indices"
    );
    assert_eq!(
        entry_indices, exit_indices,
        "Entry and Exit must agree on which SSAs the batch covered"
    );

    // The deposit data the Exit's pool produced reaches both sides, still attached to the SSA it was
    // produced for: the Entry rebuilt it from the `SsaRequest`, the Exit recalled what it sent.
    for quota in entry_cycles.iter().chain(exit_cycles.iter()) {
        let ssa_index = quota.ssa_id.ssa_index();
        assert_eq!(
            PixAddressId::new(quota.ssa_id.pseudonym(), ssa_index),
            quota.deposit_data.id,
            "deposit data must identify the SSA it belongs to"
        );
        assert_eq!(
            vec![ssa_index.get() as u8; 4].into_boxed_slice(),
            quota.deposit_data.data,
            "deposit data of SSA {ssa_index} must be the payload its own index was answered with"
        );
    }

    // Distinct deposit addresses: each entry of the batch is its own cycle, hence its own deposit.
    for (i, entry) in entry_cycles.iter().enumerate() {
        assert_eq!(
            entry.deposit_address, exit_cycles[i].deposit_address,
            "Entry and Exit must derive the same deposit address for cycle {i}"
        );
        assert_eq!(
            entry.quota_per_ssa, exit_cycles[i].quota_per_ssa,
            "Entry and Exit must agree on the quota for cycle {i}"
        );
        for (j, other) in entry_cycles.iter().enumerate().skip(i + 1) {
            assert_ne!(
                entry.deposit_address, other.deposit_address,
                "cycles {i} and {j} of the batch must have distinct deposit addresses"
            );
        }
    }

    alice_session.close().await?;
    for ah in ahs {
        ah.abort();
    }
    alice_sender.close_channel();
    bob_sender.close_channel();
    alice_handle.await??;
    bob_handle.await??;

    Ok(())
}

/// An Exit batching above the Entry's cap must fail fast on a `SessionError`, not linger until its own
/// deposit kill switch fires.
///
/// The batch size is not negotiated — `StartSession.additional_data` has no room to advertise the
/// Entry's cap — so this misconfiguration is reachable and, without the `SessionError`, silent: the
/// Exit has armed its kill switches, will never receive an `SsaCommit`, and has no event that could
/// make it re-request. It would serve the Session unincentivized for
/// `ssas_per_request × (max_deposit_wait + max_ssa_delivery_time)` — 240 s at the defaults used here —
/// and then blame the deposit.
///
/// ## Steps
/// 1. Bob (Exit) batches 3 SSAs per request; Alice (Entry) accepts at most 1, so the very first request is refused.
/// 2. Both transports relay Start protocol messages to the peer manager, counting `SessionError`s.
/// 3. Alice sends exactly one `SessionError` and drops her half of the Session.
/// 4. Bob's `handle_session_error` closes his half too. The 2 s bound is the whole point: Bob's kill-switch window is
///    240 s, so closing this quickly can only be the `SessionError` doing it.
#[test(tokio::test)]
async fn entry_refusing_an_oversized_batch_tears_down_both_halves_promptly() -> Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let ssa_gen_config = SsaGeneratorConfig {
        polynomials_per_ssa: 8,
        threshold: 2,
        surplus_shares: 2,
    };

    // Alice accepts 1, Bob asks for 3 — the mismatch this test is about.
    let alice_mgr = SessionManager::new(SessionManagerConfig {
        max_ssas_per_ssa_request: 1,
        ..Default::default()
    });
    let bob_mgr = SessionManager::new(SessionManagerConfig {
        pix_config: IncomingSessionPixConfig {
            quota_range: 0..=2048 * 1024 * 1024,
            ssas_per_request: 3,
            // Left at the defaults (60 s + 20 s), so the kill-switch window is 3 × 80 s = 240 s and
            // cannot be what closes the Session inside the assertions below.
            ..Default::default()
        },
        ..Default::default()
    });

    let alice_session_errors = Arc::new(AtomicUsize::new(0));
    let alice_session_errors_tx = alice_session_errors.clone();
    let bob_mgr_relay = Arc::new(bob_mgr.clone());
    let mut alice_transport = MockMsgSender::new();
    alice_transport
        .expect_send_message()
        .times(1..)
        .returning(move |_, data| {
            if msg_type(&data, StartProtocolDiscriminants::SessionError) {
                alice_session_errors_tx.fetch_add(1, Ordering::Relaxed);
            }
            let bob_mgr_relay = bob_mgr_relay.clone();
            Box::pin(async move {
                let _ = bob_mgr_relay.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                );
                Ok(())
            })
        });

    let alice_mgr_relay = Arc::new(alice_mgr.clone());
    let mut bob_transport = MockMsgSender::new();
    bob_transport
        .expect_send_message()
        .times(1..)
        .returning(move |_, data| {
            let alice_mgr_relay = alice_mgr_relay.clone();
            Box::pin(async move {
                let _ = alice_mgr_relay.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                );
                Ok(())
            })
        });

    let ssa_rec_config = SsaReconstructorConfig::default();
    let (pix_toolbox_alice, _pix_alice_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );
    let (pix_toolbox_bob, pix_bob_rx) = PixToolbox::new(
        SsaShareGenerator::new(ssa_gen_config).into(),
        SsaReconstructor::new(ssa_rec_config).into(),
    );
    // Bob is the Exit: without a pool answering his deposit-data request, his SSA request waits out
    // `DEPOSIT_DATA_REQUEST_TIMEOUT` and the refusal this test times would land after the deadline it
    // asserts. The events themselves are not read here — only the answering is needed.
    let _pix_bob_rx = answering_deposit_pool(pix_bob_rx, |_| Vec::new());

    let mut ahs = Vec::new();
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, Some(pix_toolbox_alice))?);

    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, Some(pix_toolbox_bob))?);

    let _notifications = tokio::spawn(async move {
        pin_mut!(new_session_rx_bob);
        while let Some(_session) = new_session_rx_bob.next().await {}
    });

    // Establishment itself may or may not complete before the refusal lands — the refusal is racing
    // `new_session`'s return, and either interleaving is fine. The end state is what matters.
    let established = tokio::time::timeout(
        Duration::from_secs(5),
        alice_mgr.new_session(
            bob_peer,
            SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            SessionClientConfig {
                pseudonym: alice_pseudonym.into(),
                capabilities: Capability::NoRateControl | Capability::Segmentation | Capability::UsePIX,
                surb_management: None,
                pix_ssa_quota: Some(PixParams::try_from_config::<HoprPixSpec>(&ssa_gen_config)?),
                return_path_options: RoutingOptions::Hops(1.try_into()?),
                ..Default::default()
            },
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;
    tracing::info!(?established, "new_session returned");

    // Both halves must be gone well inside Bob's 240 s kill-switch window.
    let closed = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if alice_mgr.active_sessions().is_empty() && bob_mgr.active_sessions().is_empty() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    assert!(
        closed.is_ok(),
        "both halves must close on the refusal, not on the deposit timeout — entry: {:?}, exit: {:?}",
        alice_mgr.active_sessions(),
        bob_mgr.active_sessions()
    );

    assert_eq!(
        1,
        alice_session_errors.load(Ordering::Relaxed),
        "the Entry must tell the Exit exactly once why the batch was refused"
    );

    for ah in ahs {
        ah.abort();
    }
    alice_sender.close_channel();
    bob_sender.close_channel();
    alice_handle.await??;
    bob_handle.await??;

    Ok(())
}
