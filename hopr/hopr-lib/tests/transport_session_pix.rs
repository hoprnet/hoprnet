//! End-to-end PIX multi-cycle session tests.
//!
//! Establishes a session between Entry and Exit with PIX enabled, keeps
//! symmetric traffic flowing in the background, and observes the PIX event cycle
//! repeat multiple times:
//!
//!   1. [Entry] `NewDepositAddress`   — deposit address generated
//!   2. [Exit]  `DepositAddressReceived` — deposit needed, notifier provided
//!   3. [Test]  Signal deposit via notifier
//!   4. [Exit]  `PrivateKeyRecovered` — quota exhausted, key recovered → SessionManager requests next SSA → goto 1
//!
//! [`capture_n_hop_pix_session`] runs that loop over 1, 2 and 3 hops with one SSA per request.
//! [`batched_ssa_request_drives_pix_cycles`] runs it over 1 hop with the Exit asking for several SSAs
//! per request, so each turn of the loop above covers a whole batch.

use hopr_lib::testing::fixtures::{
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, build_role_cluster, chain_propagation_delay,
};
#[cfg(feature = "session-client")]
use {
    anyhow::Context,
    futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt},
    hopr_api::types::primitive::prelude::HoprBalance,
    hopr_chain_connector::blokli_client::BlokliQueryClient,
    hopr_lib::{
        HoprSessionClientConfig,
        api::node::{
            HasChainApi, HasExitIncentivization, HoprSessionClientOperations, IncentiveChannelOperations, PixEvent,
        },
        exports::{
            network::types::prelude::{IpOrHost, SealedHost},
            transport::session::IncomingSessionPixConfig,
            transport::{SessionCapability, SessionTarget},
        },
    },
    rstest::rstest,
    serial_test::serial,
    std::{str::FromStr, time::Duration},
};

const FUNDING_AMOUNT: &str = "15000 wxHOPR";

// PIX params: 8 polys × (2 + 2) shares × one packet payload per share, per SSA cycle.
const PIX_POLYS: u16 = 8;
const PIX_SHARES: u8 = 2;

/// Surplus shares the Entry is configured with.
///
/// Set explicitly rather than left to derive, because every surplus share is another round-trip
/// packet — see the comment on `additional_shares` below — and because a value that differs from
/// what the derivation would produce is what makes it visible whether the surplus really crossed
/// the wire. At `PIX_SHARES = 2` the derived surplus would be zero, which would test nothing.
const PIX_SURPLUS: u8 = 2;

/// The three above as the Session asks for them.
///
/// `const` rather than built at the call site so the range check runs at compile time — the values
/// are constants, so a typo here should not need a cluster to boot before it is noticed.
const PIX_PARAMS: hopr_lib::PixParams =
    match hopr_lib::PixParams::try_new(PIX_POLYS, PIX_SHARES, PIX_SURPLUS, hopr_lib::LOCAL_PIX_SUITE) {
        Ok(params) => params,
        Err(_) => panic!("test PIX parameters must be within the protocol ranges"),
    };

/// Quota one SSA cycle costs, as the Exit computes it when deciding whether to accept the Session.
///
/// Mirrors `pix_params_to_quota`: `polys × (shares + surplus) × HoprPacket::PAYLOAD_SIZE`. Derived
/// rather than written as a literal because the payload size is a build-time constant that moves —
/// a hard-coded ceiling here does not fail loudly when it goes stale, it just makes every PIX
/// session in this file get rejected with `UnacceptablePixParams` after a cluster has booted.
const PIX_QUOTA_PER_SSA: u64 = PIX_POLYS as u64
    * (PIX_SHARES as u64 + PIX_SURPLUS as u64)
    * hopr_lib::exports::transport::PACKET_PAYLOAD_SIZE as u64;

/// Number of SSAs the Exit packs into one `SsaRequest` in [`batched_ssa_request_drives_pix_cycles`].
///
/// Deliberately above the Entry's default cap of 2, so the test also proves that
/// `pix.max_ssas_per_request` is really plumbed from node configuration through to the
/// `SessionManager`: a batch of 2 would be accepted even by an Entry that ignored the knob entirely,
/// whereas a batch of 3 is refused outright unless the raised cap takes effect.
const SSA_BATCH: usize = 3;

/// Budget for observing two full SSA batches, measured from a live session.
///
/// Generous next to the ~3 cycles of traffic it actually takes — the 1-hop
/// [`capture_n_hop_pix_session`] case completes 3 cycles well inside a minute — because the point of
/// the bound is not to be tight. It is to turn "the Exit stopped requesting SSAs" into a named
/// failure instead of a bare `rstest` timeout that says only that the test did not finish.
#[cfg(feature = "session-client")]
#[allow(unexpected_cfgs)]
const BATCH_OBSERVATION_BUDGET: Duration = if cfg!(coverage) {
    Duration::from_secs(300)
} else {
    Duration::from_secs(150)
};

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[case(2)]
#[case(3)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// n-hop PIX multi-cycle session test.
///
/// Creates a (n+2)-node role-typed cluster (Entry, N relays, Exit) where each
/// node is built with the correct transport role. The Exit accepts tiny PIX
/// quotas. Keeps symmetric 32-byte traffic flowing Entry↔Exit while observing
/// the PIX event cycle repeat 3 times.
async fn capture_n_hop_pix_session(#[case] hops: usize) -> anyhow::Result<()> {
    // 2-hop and 3-hop tests are too slow under coverage instrumentation
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // ── Role-typed cluster: Entry + N relays + Exit ─────────────────────────
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            // Entry needs PIX global config matching session-negotiated (2,2)
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: Some(PIX_SURPLUS as usize),
                ..Default::default()
            }),
            ..Default::default()
        }, // Entry: win_prob=1.0
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops], // N relays: win_prob=0.2
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                quota_range: 0..=PIX_QUOTA_PER_SSA,
                enforce_pix: false,
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                ..Default::default()
            }),
            idle_timeout_ms: Duration::from_secs(90).as_millis() as u64,
            ..Default::default()
        }, /* Exit: win_prob=1.0, custom PIX
                                                                                  * config */
    )
    .await?;

    // ── Open bidirectional channels along the relay path ───────────────────
    tracing::info!("opening channels");
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;

    // Helper macro: open channel from `$from` to `$to` using IncentiveChannelOperations
    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    // Forward: Entry → Relay[0] → Relay[1] → ... → Exit
    open_chan!(cluster.entry, cluster.relays[0]);
    for i in 0..hops.saturating_sub(1) {
        open_chan!(cluster.relays[i], cluster.relays[i + 1]);
    }
    open_chan!(cluster.relays[hops - 1], cluster.exit);

    // Backward: Exit → Relay[N-1] → ... → Relay[0] → Entry
    open_chan!(cluster.exit, cluster.relays[hops - 1]);
    for i in (1..hops).rev() {
        open_chan!(cluster.relays[i], cluster.relays[i - 1]);
    }
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tracing::info!("waiting for channel graph");

    // Wait for channels to propagate
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;

    tracing::info!("channel graph ready");

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    tracing::info!("subscribing to PIX events");
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // ── Establish PIX-enabled session: Entry → Exit, n-hop ────────────────
    tracing::info!("establishing PIX session");
    let routing = hops.try_into()?;
    let connect_fut = {
        let src_inner = cluster.entry.inner();
        let dst_addr = cluster.exit.address();
        let ip = IpOrHost::from_str(":0")?;
        async move {
            src_inner
                .connect_to(
                    dst_addr,
                    SessionTarget::UdpStream(SealedHost::Plain(ip)),
                    HoprSessionClientConfig {
                        forward_path: routing,
                        return_path: routing,
                        capabilities: SessionCapability::Segmentation
                            | SessionCapability::NoRateControl
                            | SessionCapability::UsePIX,
                        pseudonym: None,
                        surb_management: None,
                        max_surbs_per_data_packet: 1,
                        pix_ssa_quota: Some(PIX_PARAMS),
                        flow_control: None,
                        max_frames_behind_gap: None,
                    },
                )
                .await
        }
    };
    let (session, _) = tokio::time::timeout(Duration::from_secs(120), connect_fut)
        .await
        .context("session connection timed out after 120s")??;
    tracing::info!("session established");

    // ── Background data task: keep traffic flowing symmetrically ──────────
    let bg_handle = tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            let result = tokio::time::timeout(Duration::from_secs(10), async {
                wr.write_all(&msg).await?;
                wr.flush().await?;
                let mut echoed = vec![0u8; 32];
                rd.read_exact(&mut echoed).await?;
                anyhow::Ok(echoed)
            })
            .await;
            match result {
                Ok(Ok(_echoed)) => {}
                Ok(Err(e)) => {
                    tracing::warn!("bg task failed: {e:?}");
                    break;
                }
                Err(_) => {
                    tracing::warn!("bg task timed out");
                    break;
                }
            }
        }
        tracing::info!("bg task exited");
    });

    // ── Observe PIX event cycles ──────────────────────────────────────────
    let target_cycles = 3u32;
    let mut new_deposit_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut deposit_received_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut pk_recovered_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();

    loop {
        tokio::select! {
            Some(event) = entry_events.next() => {
                match event {
                    PixEvent::NewDepositAddress(data) => {
                        assert!(
                            !new_deposit_ids.contains(&data.id),
                            "duplicate NewDepositAddress for same SSA — expected distinct cycles, got {:?}",
                            data.id,
                        );
                        new_deposit_ids.push(data.id);
                        tracing::info!(id = ?data.id, quota = data.quota, "Entry: NewDepositAddress");
                    }
                    other => {
                        anyhow::bail!("unexpected Entry PixEvent: {other:?}");
                    }
                }
            }
            Some(event) = exit_events.next() => {
                match event {
                    PixEvent::DepositAddressReceived(data) => {
                        tracing::info!(id = ?data.id, quota = data.quota, "Exit: DepositAddressReceived");
                        // Signal deposit immediately to abort the kill switch
                        if let Some(mut notifier) = data.deposit_updated {
                            notifier
                                .send((data.id, HoprBalance::new_base(1)))
                                .await
                                .context("failed to signal deposit via notifier")?;
                            tracing::info!(id = ?data.id, "deposit signaled");
                        }
                        deposit_received_ids.push(data.id);
                    }
                    PixEvent::PrivateKeyRecovered(data) => {
                        assert!(
                            !pk_recovered_ids.contains(&data.id),
                            "duplicate PrivateKeyRecovered for same SSA — expected distinct cycles, got {:?}",
                            data.id,
                        );
                        pk_recovered_ids.push(data.id);
                        tracing::info!(count = pk_recovered_ids.len(), id = ?data.id, "Exit: PrivateKeyRecovered");
                    }
                    other => {
                        anyhow::bail!("unexpected Exit PixEvent: {other:?}");
                    }
                }
            }
        }

        let completed = new_deposit_ids
            .iter()
            .filter(|id| deposit_received_ids.contains(id) && pk_recovered_ids.contains(id))
            .count();
        if completed >= target_cycles as usize {
            tracing::info!(target_cycles, "all PIX cycles completed");
            break;
        }
    }

    // ── Assert lifecycle SSA ID correlation ───────────────────────────────
    // Every completed SSA cycle must pass through all three lifecycle stages
    // with the same ID: Entry generates a deposit address → Exit observes it
    // → Exit recovers the private key.
    let completed = new_deposit_ids
        .iter()
        .filter(|id| deposit_received_ids.contains(id) && pk_recovered_ids.contains(id))
        .count();
    assert!(
        completed >= target_cycles as usize,
        "expected at least {target_cycles} fully correlated SSA cycles (ID seen in: NewDepositAddress, \
         DepositAddressReceived, AND PrivateKeyRecovered), got {completed}. new_deposit_ids={new_deposit_ids:?}, \
         deposit_received_ids={deposit_received_ids:?}, pk_recovered_ids={pk_recovered_ids:?}",
    );

    // ── Stop background data task ─────────────────────────────────────────
    bg_handle.abort();

    tracing::info!(hops, "PIX multi-cycle session test PASSED");
    Ok(())
}

#[cfg(feature = "session-client")]
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// 1-hop PIX session in which the Exit requests [`SSA_BATCH`] SSAs per `SsaRequest`.
///
/// The unit tests and `hopr-transport-session`'s own integration tests already pin a batch onto a
/// single `SsaRequest` against mocked transports. What only a cluster shows is that the batch
/// survives the real path: `SSA_BATCH` commitment sets burst back through a relay over QUIC into the
/// Exit's *bounded* Start-protocol ingress channel, and every one of them has to land. A dropped
/// `SsaCommit` has no NACK, so an ingress channel that was not sized for the batch would lose a
/// cycle silently here and the Session would die on a deposit timeout minutes later.
///
/// Three properties are checked, each regressing differently:
///
///  1. **The batch is allocated up front** — at least `SSA_BATCH` deposit addresses reach the Exit before it recovers
///     its first private key. Unbatched, the Exit learns of the next address only once the current cycle is nearly
///     recovered, so this count would be one or two, never three.
///  2. **Batch N+1 follows batch N** — enforced by [`BATCH_OBSERVATION_BUDGET`] rather than an `assert!`, since the
///     failure mode is a stall, not a wrong value. Only the *last* index of a batch clears the stale-cycle guard in
///     `request_next_ssa`, so an off-by-one there leaves the Session with no further SSAs and it quietly stops rolling.
///  3. **Indices are contiguous and addresses unique** across both batches — a batch is allocated as `first .. first +
///     batch`, and a wrapped or reused index would collide with a live cycle.
async fn batched_ssa_request_drives_pix_cycles() -> anyhow::Result<()> {
    // ── Cluster: Entry + 1 relay + Exit, both PIX sides raised to SSA_BATCH ─
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: Some(PIX_SURPLUS as usize),
                // Must be raised in step with the Exit below: the batch size is not negotiated, and
                // an Entry left at its default of 2 refuses a batch of 3 outright.
                max_ssas_per_request: SSA_BATCH,
                ..Default::default()
            }),
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB)],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                quota_range: 0..=PIX_QUOTA_PER_SSA,
                enforce_pix: false,
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                ssas_per_request: SSA_BATCH,
            }),
            idle_timeout_ms: Duration::from_secs(90).as_millis() as u64,
            ..Default::default()
        },
    )
    .await?;

    // ── Open bidirectional channels Entry ↔ relay ↔ Exit ───────────────────
    tracing::info!("opening channels");
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;

    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    open_chan!(cluster.entry, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.exit);
    open_chan!(cluster.exit, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tracing::info!("waiting for channel graph");
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;
    tracing::info!("channel graph ready");

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // ── Establish the PIX-enabled session, 1 hop ──────────────────────────
    tracing::info!("establishing PIX session");
    let routing = 1usize.try_into()?;
    let connect_fut = {
        let src_inner = cluster.entry.inner();
        let dst_addr = cluster.exit.address();
        let ip = IpOrHost::from_str(":0")?;
        async move {
            src_inner
                .connect_to(
                    dst_addr,
                    SessionTarget::UdpStream(SealedHost::Plain(ip)),
                    HoprSessionClientConfig {
                        forward_path: routing,
                        return_path: routing,
                        capabilities: SessionCapability::Segmentation
                            | SessionCapability::NoRateControl
                            | SessionCapability::UsePIX,
                        pseudonym: None,
                        surb_management: None,
                        max_surbs_per_data_packet: 1,
                        pix_ssa_quota: Some(PIX_PARAMS),
                        flow_control: None,
                        max_frames_behind_gap: None,
                    },
                )
                .await
        }
    };
    let (session, _) = tokio::time::timeout(Duration::from_secs(120), connect_fut)
        .await
        .context("session connection timed out after 120s")??;
    tracing::info!("session established");

    // ── Background data task: burn through the SSA quota ───────────────────
    let bg_handle = tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            let result = tokio::time::timeout(Duration::from_secs(10), async {
                wr.write_all(&msg).await?;
                wr.flush().await?;
                let mut echoed = vec![0u8; 32];
                rd.read_exact(&mut echoed).await?;
                anyhow::Ok(echoed)
            })
            .await;
            match result {
                Ok(Ok(_echoed)) => {}
                Ok(Err(e)) => {
                    tracing::warn!("bg task failed: {e:?}");
                    break;
                }
                Err(_) => {
                    tracing::warn!("bg task timed out");
                    break;
                }
            }
        }
        tracing::info!("bg task exited");
    });

    // ── Observe two consecutive batches ───────────────────────────────────
    let mut entry_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut exit_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut exit_addresses = Vec::new();
    let mut recovered_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    // Latched on the first recovery: how many addresses the Exit had been handed by the time it
    // finished its first cycle. This is the establishment batch, counted from the Exit's own stream
    // so no cross-stream ordering is involved.
    let mut addresses_before_first_recovery: Option<usize> = None;

    let observe = async {
        loop {
            tokio::select! {
                event = entry_events.next() => {
                    match event.context("Entry PIX event stream ended while the session was live")? {
                        PixEvent::NewDepositAddress(data) => {
                            assert!(
                                !entry_ids.contains(&data.id),
                                "duplicate NewDepositAddress for {:?} — every cycle in a batch needs its own SSA",
                                data.id,
                            );
                            entry_ids.push(data.id);
                            tracing::info!(id = ?data.id, quota = data.quota, "Entry: NewDepositAddress");
                        }
                        other => anyhow::bail!("unexpected Entry PixEvent: {other:?}"),
                    }
                }
                event = exit_events.next() => {
                    match event.context("Exit PIX event stream ended while the session was live")? {
                        PixEvent::DepositAddressReceived(data) => {
                            tracing::info!(id = ?data.id, quota = data.quota, "Exit: DepositAddressReceived");
                            assert!(
                                !exit_ids.contains(&data.id),
                                "duplicate DepositAddressReceived for {:?} — a reused SSA index would \
                                 collide with a live cycle",
                                data.id,
                            );
                            assert!(
                                !exit_addresses.contains(&data.address),
                                "deposit address for {:?} was already used by an earlier cycle — each SSA \
                                 in a batch must get its own",
                                data.id,
                            );
                            exit_ids.push(data.id);
                            exit_addresses.push(data.address);
                            // Signal the deposit immediately so this cycle's kill switch is aborted.
                            if let Some(mut notifier) = data.deposit_updated {
                                notifier
                                    .send((data.id, HoprBalance::new_base(1)))
                                    .await
                                    .context("failed to signal deposit via notifier")?;
                            }
                        }
                        PixEvent::PrivateKeyRecovered(data) => {
                            addresses_before_first_recovery.get_or_insert(exit_ids.len());
                            assert!(
                                !recovered_ids.contains(&data.id),
                                "duplicate PrivateKeyRecovered for {:?}",
                                data.id,
                            );
                            recovered_ids.push(data.id);
                            tracing::info!(count = recovered_ids.len(), id = ?data.id, "Exit: PrivateKeyRecovered");
                        }
                        other => anyhow::bail!("unexpected Exit PixEvent: {other:?}"),
                    }
                }
            }

            // Twice the batch size proves a *second* batch was requested, and `SSA_BATCH` recoveries
            // prove the first batch's cycles actually reconstructed rather than merely being handed out.
            if exit_ids.len() >= 2 * SSA_BATCH && recovered_ids.len() >= SSA_BATCH {
                break;
            }
        }
        anyhow::Ok(())
    };

    tokio::time::timeout(BATCH_OBSERVATION_BUDGET, observe).await.context(
        "timed out observing two SSA batches — the Exit most likely stopped requesting SSAs after the first batch, \
         which is how a stale-cycle off-by-one manifests",
    )??;

    // ── 1. The whole batch is allocated before the first cycle completes ──
    let before_first_recovery = addresses_before_first_recovery.context("the Exit never recovered a key")?;
    assert!(
        before_first_recovery >= SSA_BATCH,
        "the Exit was told about only {before_first_recovery} deposit address(es) before it recovered its first key; \
         a batch of {SSA_BATCH} is allocated up front, so it should already know all {SSA_BATCH}. Getting one or two \
         here means the batch size did not take effect. exit_ids={exit_ids:?}",
    );

    // ── 2. Contiguous indices from 1, across the batch boundary ───────────
    let mut indices: Vec<u32> = exit_ids.iter().map(|(_, index)| index.get()).collect();
    indices.sort_unstable();
    assert_eq!(
        indices,
        (1..=exit_ids.len() as u32).collect::<Vec<_>>(),
        "SSA indices must be contiguous from 1 both within a batch and from one batch to the next",
    );

    // ── 3. Every recovered cycle passed through all three stages ──────────
    let fully_correlated = recovered_ids
        .iter()
        .filter(|id| entry_ids.contains(id) && exit_ids.contains(id))
        .count();
    assert!(
        fully_correlated >= SSA_BATCH,
        "expected at least {SSA_BATCH} SSA cycles seen at all three stages (NewDepositAddress, DepositAddressReceived \
         AND PrivateKeyRecovered), got {fully_correlated}. entry_ids={entry_ids:?}, exit_ids={exit_ids:?}, \
         recovered_ids={recovered_ids:?}",
    );

    bg_handle.abort();

    tracing::info!(
        batch = SSA_BATCH,
        addresses = exit_ids.len(),
        recovered = recovered_ids.len(),
        "batched PIX session test PASSED"
    );
    Ok(())
}
