use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{Sink, SinkExt, future::AbortHandle};
use futures_time::future::FutureExt;
use hopr_api::types::{
    internal::{prelude::HoprPseudonym, routing::DestinationRouting},
    primitive::balance::HoprBalance,
};
use hopr_crypto_packet::HoprPixSpec;
use hopr_protocol_pix::{SsaCommitmentGuard, SsaId, SsaReconstructor};
use hopr_protocol_start::KeepAliveMessage;
use tracing::{error, info, warn};

use super::{ClosedSessionOffer, DrainOutcome, DrainResult, DrainStopReason, SurbDrainConfig};
use crate::HoprStartProtocol;

/// Handle to a running drain task.
pub(crate) struct DrainTaskHandle {
    /// Abort handle to kill the task.
    pub(crate) abort_handle: AbortHandle,
    /// Sender for forwarding PIX events to this drain task.
    pub(crate) event_tx: crossfire::MTx<crossfire::mpsc::List<DrainEvent>>,
}

/// Events forwarded to a running drain task.
#[derive(Debug, Clone)]
pub(crate) enum DrainEvent {
    /// Progress on a specific SSA (recovered share).
    SsaRecovered(SsaId<HoprPseudonym>),
}

/// Internal accumulator for one SSA being drained.
struct SsaDrainTarget {
    guard: Option<SsaCommitmentGuard<HoprPixSpec>>,
    deficit: u64,
    baseline_invalid: u64,
}

/// Spawn a drain task for a closed session.
///
/// Returns the abort handle and event sender so the caller can wire shutdown
/// and event forwarding.
pub(crate) fn spawn_drain_task(
    msg_sender: impl Sink<
        (DestinationRouting, hopr_protocol_app::v1::ApplicationDataOut),
        Error = impl std::error::Error + Send + Sync + 'static,
    > + Clone
    + Unpin
    + Send
    + 'static,
    cfg: SurbDrainConfig,
    offer: ClosedSessionOffer,
    surb_count: Arc<dyn Fn(&HoprPseudonym) -> usize + Send + Sync>,
    _packet_price: Arc<dyn Fn() -> HoprBalance + Send + Sync>,
    reconstructor: Arc<SsaReconstructor<HoprPixSpec>>,
    max_packets: u64,
    deficits: Vec<(usize, u64)>,
    on_outcome: crossfire::MTx<crossfire::mpsc::List<DrainOutcome>>,
) -> DrainTaskHandle {
    let (event_tx, event_rx) = crossfire::mpsc::unbounded_async();
    let abort_handle = AbortHandle::new_pair().0;

    hopr_utils::runtime::prelude::spawn(run_drain(
        msg_sender,
        cfg,
        offer,
        surb_count,
        reconstructor,
        max_packets,
        deficits,
        on_outcome,
        event_rx,
        abort_handle.clone(),
    ));

    DrainTaskHandle { abort_handle, event_tx }
}

#[allow(clippy::too_many_arguments)]
async fn run_drain(
    mut msg_sender: impl Sink<
        (DestinationRouting, hopr_protocol_app::v1::ApplicationDataOut),
        Error = impl std::error::Error + Send + Sync + 'static,
    > + Unpin,
    cfg: SurbDrainConfig,
    offer: ClosedSessionOffer,
    surb_count: Arc<dyn Fn(&HoprPseudonym) -> usize + Send + Sync>,
    reconstructor: Arc<SsaReconstructor<HoprPixSpec>>,
    max_packets: u64,
    deficits: Vec<(usize, u64)>,
    on_outcome: crossfire::MTx<crossfire::mpsc::List<DrainOutcome>>,
    event_rx: crossfire::AsyncRx<crossfire::mpsc::List<DrainEvent>>,
    drain_abort: AbortHandle,
) {
    let session_id = offer.session_id;
    let pseudonym: HoprPseudonym = session_id.into();
    let routing = offer.routing;
    let deadline = Instant::now() + cfg.max_drain_time;
    let ack_grace = cfg.ack_grace;

    // Build per-SSA targets from the handover.
    let mut targets: Vec<SsaDrainTarget> = offer
        .ssas
        .into_iter()
        .enumerate()
        .map(|(i, ssa)| {
            let baseline_invalid = reconstructor
                .drain_snapshot(ssa.guard.ssa_id())
                .map(|s| s.invalid_total)
                .unwrap_or(0);
            SsaDrainTarget {
                guard: Some(ssa.guard),
                deficit: deficits.iter().find(|(idx, _)| *idx == i).map(|(_, d)| *d).unwrap_or(0),
                baseline_invalid,
            }
        })
        .collect();

    let mut sent: u64 = 0;
    let mut last_progress = Instant::now();
    let mut ssas_recovered: u32 = 0;
    let mut event_buf = Vec::new();

    let stop_reason = 'drain: loop {
        // Check abort.
        if drain_abort.is_aborted() {
            break DrainStopReason::Shutdown;
        }

        // Deadline.
        if Instant::now() >= deadline {
            break DrainStopReason::DeadlineReached;
        }

        // Budget.
        if sent >= max_packets {
            break DrainStopReason::BudgetExhausted;
        }

        // SURB supply.
        if surb_count(&pseudonym) == 0 {
            break DrainStopReason::SurbsExhausted;
        }

        // Drain all pending events (non-blocking).
        event_buf.clear();
        while let Ok(evt) = event_rx.try_recv() {
            event_buf.push(evt);
        }
        for DrainEvent::SsaRecovered(ssa_id) in &event_buf {
            for t in &mut targets {
                if t.guard.as_ref().map(|g| g.ssa_id()) == Some(ssa_id) && t.deficit > 0 {
                    t.deficit = 0;
                    ssas_recovered += 1;
                    last_progress = Instant::now();
                }
            }
        }

        // Check reconstructor for any share progress or faults.
        let mut made_progress = false;
        for t in &mut targets {
            if t.deficit == 0 {
                continue;
            }
            if let Some(guard) = t.guard.as_ref() {
                if let Some(snap) = reconstructor.drain_snapshot(guard.ssa_id()) {
                    // Zero-tolerance for unverifiable shares.
                    if snap.invalid_total > t.baseline_invalid {
                        break 'drain DrainStopReason::UnverifiableShare;
                    }
                    let useful = snap.progress.useful_shares;
                    let target = snap.progress.target_useful_shares;
                    if useful >= target {
                        t.deficit = 0;
                        ssas_recovered += 1;
                        made_progress = true;
                    } else {
                        let old = t.deficit;
                        t.deficit = target - useful;
                        if t.deficit < old {
                            made_progress = true;
                        }
                    }
                }
            }
        }

        // All recovered?
        if targets.iter().all(|t| t.deficit == 0) {
            break DrainStopReason::AllRecovered;
        }

        if made_progress {
            last_progress = Instant::now();
        } else if last_progress.elapsed() >= ack_grace {
            break DrainStopReason::NoProgress;
        }

        // Send one keep-alive packet.
        let keepalive = HoprStartProtocol::KeepAlive(KeepAliveMessage {
            session_id: session_id.into(),
            flags: Default::default(),
            additional_data: sent,
        });
        let app_data: hopr_protocol_app::v1::ApplicationData = match keepalive.try_into() {
            Ok(d) => d,
            Err(e) => {
                error!(%session_id, "failed to convert KeepAlive to ApplicationData: {e:?}");
                break DrainStopReason::Shutdown;
            }
        };

        match msg_sender
            .send((
                routing.clone(),
                hopr_protocol_app::v1::ApplicationDataOut::with_no_packet_info(app_data),
            ))
            .timeout(futures_time::time::Duration::from_millis(200))
            .await
        {
            Ok(Ok(())) => {
                sent += 1;
                // Small rate-limit delay to avoid flooding.
                futures_time::task::sleep(Duration::from_millis(1000 / cfg.drain_rate_packets_per_sec as u64).into())
                    .await;
            }
            Ok(Err(e)) => {
                warn!(%session_id, error = %e, "drain send failed");
            }
            Err(_) => {
                warn!(%session_id, "drain send timed out");
            }
        }
    };

    // On stop: drop remaining guards (triggers retire_ssa).
    drop(targets);

    let outcome = DrainOutcome {
        session_id,
        result: DrainResult::Finished(stop_reason),
        packets_sent: sent,
        ssas_recovered,
    };

    info!(%session_id, ?outcome.result, sent, ssas_recovered, "drain finished");

    let _ = on_outcome.try_send(outcome);
}
