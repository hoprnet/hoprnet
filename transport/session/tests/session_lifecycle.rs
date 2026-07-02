//! Integration tests for `SessionManager` session establishment and lifecycle.

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
use hopr_protocol_start::StartProtocolDiscriminants;
use hopr_transport_session::{
    ApplicationDataIn, Capability, DestinationRouting, HoprStartProtocol, SessionClientConfig, SessionManager,
    SessionManagerConfig, SessionTarget, SurbBalancerConfig,
};
use hopr_utils::network_types::prelude::SealedHost;
use test_log::test;

use crate::common::{MockMsgSender, mock_packet_planning, msg_type, start_msg_match};

#[test(tokio::test)]
async fn session_manager_should_follow_start_protocol_to_establish_new_session_and_close_it() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let alice_mgr = SessionManager::new(Default::default());
    let bob_mgr = SessionManager::new(Default::default());

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let bob_mgr_for_alice = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::info!("alice sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr = bob_mgr_for_alice.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Bob sends the SessionEstablished message
    let alice_mgr_for_bob = Arc::new(alice_mgr.clone());
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            tracing::info!("bob sends {}", data.data.application_tag);
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let alice_mgr = alice_mgr_for_bob.clone();
            Box::pin(async move {
                alice_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends the terminating segment to close the Session
    let bob_mgr_for_alice_seg = Arc::new(bob_mgr.clone());
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
            let bob_mgr = bob_mgr_for_alice_seg.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    let mut ahs = Vec::new();

    // Start Alice
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?);
    assert!(alice_mgr.is_started());

    // Start Bob
    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?);
    assert!(bob_mgr.is_started());

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target.clone()),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation,
                    surb_management: None,
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;
    let (alice_session, bob_session) = {
        let (r, o) = result;
        (r?, o.expect("bob must get an incoming session"))
    };

    let mut alice_session = alice_session;

    assert_eq!(
        alice_session.config().capabilities,
        Capability::Segmentation | Capability::NoRateControl
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

    tokio::time::sleep(Duration::from_millis(100)).await;
    alice_session.close().await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(matches!(
        alice_mgr.ping_session(alice_session.id()).await,
        Err(hopr_transport_session::errors::TransportSessionError::Manager(
            hopr_transport_session::errors::SessionManagerError::NonExistingSession
        ))
    ));

    futures::stream::iter(ahs)
        .for_each(|ah| async move { ah.abort() })
        .await;

    // Cleanup: close senders and await handles
    alice_sender.close_channel();
    bob_sender.close_channel();
    let _ = alice_handle.await;
    let _ = bob_handle.await;

    Ok(())
}

#[test(tokio::test)]
async fn session_manager_should_close_idle_session_automatically() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let cfg = SessionManagerConfig {
        idle_timeout: Duration::from_millis(200),
        ..Default::default()
    };

    let alice_mgr = SessionManager::new(cfg);
    let bob_mgr = SessionManager::new(Default::default());

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let bob_mgr_for_alice = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr = bob_mgr_for_alice.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Bob sends the SessionEstablished message
    let alice_mgr_for_bob = Arc::new(alice_mgr.clone());
    bob_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let alice_mgr = alice_mgr_for_bob.clone();
            Box::pin(async move {
                alice_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    let mut ahs = Vec::new();

    // Start Alice
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?);

    // Start Bob
    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?);
    assert!(bob_mgr.is_started());

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target.clone()),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation,
                    surb_management: None,
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;
    let (alice_session, bob_session) = {
        let (r, o) = result;
        (r?, o.ok_or(anyhow::anyhow!("bob must get an incoming session"))?)
    };

    assert_eq!(
        alice_session.config().capabilities,
        Capability::Segmentation | Capability::NoRateControl,
    );
    assert_eq!(
        alice_session.config().capabilities,
        bob_session.session.config().capabilities
    );
    assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

    // Let the session timeout at Alice
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(matches!(
        alice_mgr.ping_session(alice_session.id()).await,
        Err(hopr_transport_session::errors::TransportSessionError::Manager(
            hopr_transport_session::errors::SessionManagerError::NonExistingSession
        ))
    ));

    futures::stream::iter(ahs)
        .for_each(|ah| async move { ah.abort() })
        .await;

    // Cleanup: close senders and await handles
    alice_sender.close_channel();
    bob_sender.close_channel();
    let _ = alice_handle.await;
    let _ = bob_handle.await;

    Ok(())
}

#[test(tokio::test)]
async fn session_manager_should_not_allow_loopback_sessions() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let alice_mgr = SessionManager::new(Default::default());

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let alice_mgr_for_start = Arc::new(alice_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let alice_mgr = alice_mgr_for_start.clone();
            Box::pin(async move {
                alice_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends the SessionEstablished message (as Bob)
    let alice_mgr_for_est = Arc::new(alice_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let alice_mgr = alice_mgr_for_est.clone();
            Box::pin(async move {
                alice_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Start Alice
    let (new_session_tx_alice, new_session_rx_alice) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;
    assert!(alice_mgr.is_started());

    let alice_session = alice_mgr
        .new_session(
            bob_peer,
            SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            SessionClientConfig {
                capabilities: None.into(),
                pseudonym: alice_pseudonym.into(),
                surb_management: None,
                ..Default::default()
            },
        )
        .await;

    tracing::info!("{alice_session:?}");
    assert!(matches!(
        alice_session,
        Err(hopr_transport_session::errors::TransportSessionError::Manager(
            hopr_transport_session::errors::SessionManagerError::Loopback
        ))
    ));

    drop(new_session_rx_alice);

    // Cleanup: close sender and await handle
    alice_sender.close_channel();
    let _ = alice_handle.await;

    Ok(())
}

#[test(tokio::test)]
async fn session_manager_should_timeout_new_session_attempt_when_no_response() -> Result<()> {
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let cfg = SessionManagerConfig {
        initiation_timeout_base: Duration::from_millis(100),
        ..Default::default()
    };

    let alice_mgr = SessionManager::new(cfg);
    let bob_mgr = SessionManager::new(Default::default());

    let mut sequence = mockall::Sequence::new();
    let mut alice_transport = MockMsgSender::new();
    let bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message, but Bob does not handle it
    alice_transport
        .expect_send_message()
        .once()
        .in_sequence(&mut sequence)
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(|_, _| Box::pin(async { Ok(()) }));

    // Start Alice
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, _alice_handle) = mock_packet_planning(alice_transport);
    alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;
    assert!(alice_mgr.is_started());

    // Start Bob
    let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
    let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
    bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?;
    assert!(bob_mgr.is_started());

    let result = alice_mgr
        .new_session(
            bob_peer,
            SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            SessionClientConfig {
                capabilities: None.into(),
                pseudonym: None,
                surb_management: None,
                ..Default::default()
            },
        )
        .await;

    assert!(matches!(
        result,
        Err(hopr_transport_session::errors::TransportSessionError::Timeout)
    ));

    Ok(())
}

#[test(tokio::test)]
async fn session_manager_should_send_keep_alive_when_ping_session_is_called() -> Result<()> {
    let alice_pseudonym = HoprPseudonym::random();
    let bob_peer: Address = (&ChainKeypair::random()).into();

    let alice_mgr = SessionManager::new(Default::default());
    let bob_mgr = SessionManager::new(Default::default());

    let mut alice_transport = MockMsgSender::new();
    let mut bob_transport = MockMsgSender::new();

    // Alice sends the StartSession message
    let bob_mgr_for_alice = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::StartSession)
                && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
        })
        .returning(move |_, data| {
            let bob_mgr = bob_mgr_for_alice.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Bob sends the SessionEstablished message
    let alice_mgr_for_bob = Arc::new(alice_mgr.clone());
    bob_transport
        .expect_send_message()
        .once()
        .withf(move |peer, data| {
            msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let alice_mgr = alice_mgr_for_bob.clone();
            Box::pin(async move {
                alice_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends a KeepAlive ping to Bob (via ping_session)
    let bob_mgr_for_keepalive = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .withf(move |peer, data| {
            start_msg_match(data, |msg| {
                matches!(msg, HoprStartProtocol::KeepAlive(ka) if ka.session_id == alice_pseudonym)
            }) && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let bob_mgr = bob_mgr_for_keepalive.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    // Alice sends the terminating segment (via alice_session.close())
    let bob_mgr_for_alice_seg = Arc::new(bob_mgr.clone());
    alice_transport
        .expect_send_message()
        .once()
        .withf(move |peer, data| {
            hopr_protocol_session::types::SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::try_from(
                data.data.plain_text.as_ref(),
            )
            .expect("must be a session message")
            .try_as_segment()
            .expect("must be a segment")
            .is_terminating()
                && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
        })
        .returning(move |_, data| {
            let bob_mgr = bob_mgr_for_alice_seg.clone();
            Box::pin(async move {
                bob_mgr.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                )?;
                Ok(())
            })
        });

    let mut ahs = Vec::new();

    // Start Alice
    let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
    let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
    ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?);
    assert!(alice_mgr.is_started());

    // Start Bob
    let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
    let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
    ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?);
    assert!(bob_mgr.is_started());

    let target = SealedHost::Plain("127.0.0.1:80".parse()?);

    pin_mut!(new_session_rx_bob);
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        futures::future::join(
            alice_mgr.new_session(
                bob_peer,
                SessionTarget::TcpStream(target),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: Capability::NoRateControl | Capability::Segmentation,
                    surb_management: None,
                    ..Default::default()
                },
            ),
            new_session_rx_bob.next(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("timeout: {e}"))?;
    let (mut alice_session, bob_session) = {
        let (r, o) = result;
        (r?, o.expect("bob must get an incoming session"))
    };

    assert_eq!(
        alice_session.config().capabilities,
        Capability::Segmentation | Capability::NoRateControl,
    );
    assert_eq!(
        alice_session.config().capabilities,
        bob_session.session.config().capabilities
    );

    // Ping the established session immediately — before any SURB balancer timer fires —
    // and verify it succeeds.
    alice_mgr.ping_session(alice_session.id()).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;
    alice_session.close().await?;

    futures::stream::iter(ahs)
        .for_each(|ah| async move { ah.abort() })
        .await;

    // Cleanup: close senders and await handles
    alice_sender.close_channel();
    bob_sender.close_channel();
    let _ = alice_handle.await;
    let _ = bob_handle.await;

    Ok(())
}
