//! HOPR-transport wiring for the Session send-window flow control.
//!
//! The algorithm itself (the AIMD [`WindowController`], the honest-clock [`DeliverySignal`], the
//! [`SupplyConstraint`] trait) lives in [`hopr_protocol_session::flow_control`]. This module supplies
//! the two HOPR-specific pieces:
//!
//! * [`SurbSupply`] — the anti-grief SURB ceiling, reading the existing atomic [`BalancerStateValues`] as a
//!   **down-only** clamp (never a signal to speed up — invariant 2).
//! * [`PacedWriter`] — wraps the Session socket's write half and admits bytes only while the window has room, parking
//!   on a keep-progress timer otherwise. It never gates the read half, so the duplex socket cannot deadlock.

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use hopr_protocol_session::flow_control::{
    Backoff, DeliveryClock, DeliverySignal, FlowControlConfig, SupplyConstraint, WindowController,
};

use crate::balancer::BalancerStateValues;

/// SURB-supply ceiling over the balancer's atomic state. **Down-only**: it can cap or shrink the
/// window (invariant 2), never open it. A healthy buffer returns no backoff — that is *not* a signal
/// to go faster; only proven delivery opens the window.
pub struct SurbSupply {
    state: Arc<BalancerStateValues>,
    /// Bytes one reply packet can carry (≈ one SURB consumed per reply). `buffer_level` SURBs thus
    /// cap the return path to `buffer_level × bytes_per_reply_packet` in-flight bytes.
    bytes_per_reply_packet: usize,
    /// Soft-backoff watermark as a fraction of the target buffer size.
    low_watermark_frac: f64,
}

impl SurbSupply {
    /// Creates a ceiling over `state`. `bytes_per_reply_packet` is the session frame/packet payload
    /// size. Soft backoff triggers below 25 % of the target buffer.
    pub fn new(state: Arc<BalancerStateValues>, bytes_per_reply_packet: usize) -> Self {
        Self {
            state,
            bytes_per_reply_packet: bytes_per_reply_packet.max(1),
            low_watermark_frac: 0.25,
        }
    }

    /// Raw SURB buffer level (for diagnostics).
    fn buffer_level(&self) -> u64 {
        self.state.buffer_level()
    }
}

impl SupplyConstraint for SurbSupply {
    fn max_admissible_inflight(&self) -> usize {
        // Balancer disabled ⇒ no SURB throttle ⇒ no ceiling (the window is then governed purely by
        // the honest delivery clock).
        if self.state.is_disabled() {
            return usize::MAX;
        }
        (self.state.buffer_level() as usize).saturating_mul(self.bytes_per_reply_packet)
    }

    fn backoff_hint(&self) -> Option<Backoff> {
        if self.state.is_disabled() {
            return None;
        }
        let level = self.state.buffer_level();
        if level == 0 {
            // Out of SURBs: collapse to the floor.
            return Some(Backoff::Hard);
        }
        let target = self.state.as_config().target_surb_buffer_size;
        if target > 0 && (level as f64) < target as f64 * self.low_watermark_frac {
            Some(Backoff::Soft)
        } else {
            // Healthy buffer: NOT a signal to open the window — only delivery does that.
            None
        }
    }
}

/// Wraps a Session socket, admitting writes only while the [`WindowController`] has room against the
/// honest delivery clock and the SURB ceiling. Reads are delegated untouched (never gated), so the
/// duplex socket cannot deadlock; the window always keeps at least `min_window_size` admissible.
///
/// `S` is `Unpin` (the boxed Session socket is), so this needs no pin projection.
pub struct PacedWriter<S> {
    inner: S,
    window: WindowController,
    clock: DeliveryClock,
    supply: SurbSupply,
    /// Keep-progress deadline: how long to park when the window is momentarily full before
    /// re-checking, and the persist-probe interval (see `stalled_parks`).
    deadline: Duration,
    /// Pending park timer, recreated per park.
    park: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    /// Consecutive keep-progress parks that saw **no** honest delivery and admitted **no** bytes.
    /// Reset to 0 on any admission or any delivery. When it reaches `persist_after` (> 0), the
    /// persist probe fires (see [`Self::admissible`]).
    stalled_parks: u32,
    /// Persist-probe threshold (consecutive no-progress parks). `0` disables the probe — the default
    /// clean behaviour; a robust profile sets it (~8). Sourced from [`FlowControlConfig`].
    persist_after: u32,
    /// Cumulative bytes admitted (diagnostics).
    sent_total: u64,
    /// `refresh_window` call counter, for throttling the diagnostic trace.
    refreshes: u64,
}

impl<S> PacedWriter<S> {
    /// Assembles a paced writer. `clock` is the honest delivery clock (fed by the reliable ack tap
    /// or return bytes); `supply` is the SURB ceiling; `cfg` seeds the window.
    pub fn new(inner: S, cfg: FlowControlConfig, clock: DeliveryClock, supply: SurbSupply) -> Self {
        Self {
            inner,
            window: WindowController::new(cfg),
            clock,
            supply,
            deadline: cfg.no_honest_deadline,
            park: None,
            stalled_parks: 0,
            persist_after: cfg.persist_stall_parks,
            sent_total: 0,
            refreshes: 0,
        }
    }

    /// Folds the latest honest delivery into the window. `cwnd` moves **only** on the honest delivery
    /// clock (up on ack, down on loss); SURB supply never touches `cwnd` — so a transient
    /// `buffer_level` dip cannot destroy the learned window (it can only clamp the effective window
    /// down via the ceiling, in [`Self::admissible`]).
    fn refresh_window(&mut self) {
        let delivered = self.clock.poll_delivered();
        self.window.apply_delivery(delivered);
        if delivered.acked_bytes > 0 || delivered.lost_bytes > 0 {
            self.stalled_parks = 0;
        }

        self.refreshes = self.refreshes.wrapping_add(1);
        if self.refreshes.is_multiple_of(256) {
            tracing::debug!(
                target: "hopr_flow_control",
                sent_total = self.sent_total,
                cwnd = self.window.window(),
                inflight = self.window.inflight(),
                raw_ceiling = self.supply.max_admissible_inflight(),
                buffer_level = self.supply.buffer_level(),
                stalled_parks = self.stalled_parks,
                "flow-control state"
            );
        }
    }

    /// Bytes admissible right now, against the live SURB ceiling.
    ///
    /// Normally this is `min(cwnd, surb_ceiling) − inflight`: the AIMD window bounds the send rate to
    /// the drain rate, and the SURB ceiling clamps it down when supply is low (never up — invariant 2).
    ///
    /// **Persist probe (opt-in, anti-deadlock).** At end-of-stream a HOPR session has no half-close,
    /// so the final frames may be acked slowly (or their retransmissions must exhaust before their
    /// bytes retire from `inflight`). Meanwhile `inflight` sits at `cwnd`, so the normal formula
    /// admits 0 and the writer would park forever — the tail deadlock the 5-sample measurement
    /// showed. When enabled (`persist_after > 0`), after that many consecutive no-progress parks we
    /// admit a bounded `min_window_size` **beyond `cwnd` but still capped by the SURB ceiling**. This is the
    /// classic TCP persist-timer escape, and it is invariant-safe: it never spends past the SURB
    /// ceiling (anti-grief intact), and a peer withholding acks can extract at most `min_window_size` per
    /// `persist_after × deadline` — it can only ever make us slower, never faster or over-spending.
    /// `persist_after == 0` (the default) disables it entirely: the verified clean behaviour.
    fn admissible(&self) -> usize {
        let ceiling = self.supply.max_admissible_inflight();
        admit_bytes(
            self.window.admissible_for(ceiling),
            self.stalled_parks,
            self.persist_after,
            self.window.inflight(),
            self.window.min_window_size(),
            ceiling,
        )
    }
}

/// Pure admission decision, including the persist probe. Extracted for unit testing.
///
/// * `normal` = the AIMD/ceiling-bounded admissible bytes. If positive, use it (no probe).
/// * Otherwise, if the persist probe is enabled (`persist_after > 0`) and `stalled_parks` has reached it, admit up to
///   `min_window_size` **beyond `inflight`**, but never past the SURB `ceiling` — the anti-deadlock persist probe.
///   `persist_after == 0` disables the probe entirely (the default clean behaviour).
fn admit_bytes(
    normal: usize,
    stalled_parks: u32,
    persist_after: u32,
    inflight: usize,
    min_window_size: usize,
    ceiling: usize,
) -> usize {
    if normal > 0 {
        return normal;
    }
    if persist_after > 0 && stalled_parks >= persist_after {
        let probe_target = inflight.saturating_add(min_window_size);
        return probe_target.min(ceiling).saturating_sub(inflight);
    }
    0
}

impl<S: futures::AsyncWrite + Unpin> futures::AsyncWrite for PacedWriter<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        // An empty write carries no bytes and must never be flow-controlled (otherwise it would park
        // on the timer and return `Pending` forever). Complete it immediately.
        if buf.is_empty() {
            return Pin::new(&mut self.get_mut().inner).poll_write(cx, buf);
        }
        let this = self.get_mut();
        loop {
            this.refresh_window();
            let admissible = this.admissible();
            if admissible == 0 {
                // Window full and not yet stalled long enough to persist: park on the keep-progress
                // timer, count the stall, then re-check.
                let deadline = this.deadline;
                let fut = this.park.get_or_insert_with(|| Box::pin(sleep(deadline)));
                match fut.as_mut().poll(cx) {
                    Poll::Ready(()) => {
                        this.park = None;
                        this.stalled_parks = this.stalled_parks.saturating_add(1);
                        continue; // re-evaluate admission (delivery/ceiling/persist may have moved)
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            let to_write = buf.len().min(admissible);
            return match Pin::new(&mut this.inner).poll_write(cx, &buf[..to_write]) {
                Poll::Ready(Ok(n)) => {
                    this.window.on_sent(n);
                    this.sent_total = this.sent_total.wrapping_add(n as u64);
                    this.stalled_parks = 0; // forward progress — reset the persist counter
                    Poll::Ready(Ok(n))
                }
                other => other,
            };
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_close(cx)
    }
}

impl<S: futures::AsyncRead + Unpin> futures::AsyncRead for PacedWriter<S> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        // Reads are never gated by the send window.
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}

/// Runtime-agnostic sleep returning a `()`-future (via `futures-time`).
fn sleep(dur: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        let _ = futures_time::task::sleep(dur.into()).await;
    })
}

#[cfg(test)]
mod tests {
    use futures::AsyncWriteExt;
    use hopr_protocol_session::flow_control::DeliveryMeter;

    use super::*;

    fn balancer(target: u64, buffer_level: u64) -> Arc<BalancerStateValues> {
        let cfg = crate::SurbBalancerConfig {
            target_surb_buffer_size: target,
            max_surbs_per_sec: 5_000,
            ..Default::default()
        };
        let state = Arc::new(BalancerStateValues::from(cfg));
        state
            .buffer_level
            .store(buffer_level, std::sync::atomic::Ordering::Relaxed);
        state
    }

    // ---- Persist probe (anti-deadlock at the un-acked tail), configurable ----

    const MIN_WIN: usize = 4_096;
    const PERSIST: u32 = 8;

    #[test]
    fn persist_disabled_by_default_never_fires() {
        // persist_after == 0 (default clean profile): the probe never fires, however long we stall.
        assert_eq!(admit_bytes(0, 0, 0, 50_000, MIN_WIN, usize::MAX), 0);
        assert_eq!(admit_bytes(0, 10_000, 0, 50_000, MIN_WIN, usize::MAX), 0);
    }

    #[test]
    fn persist_inactive_when_window_has_room() {
        // Normal admissible > 0 ⇒ probe never involved, whatever the stall count / profile.
        assert_eq!(admit_bytes(10_000, 0, PERSIST, 50_000, MIN_WIN, usize::MAX), 10_000);
        assert_eq!(admit_bytes(10_000, 999, PERSIST, 50_000, MIN_WIN, usize::MAX), 10_000);
    }

    #[test]
    fn persist_holds_until_stall_threshold() {
        // Window full (normal == 0) but not yet stalled long enough ⇒ admit nothing.
        for parks in 0..PERSIST {
            assert_eq!(
                admit_bytes(0, parks, PERSIST, 50_000, MIN_WIN, usize::MAX),
                0,
                "parks={parks}"
            );
        }
    }

    #[test]
    fn persist_fires_after_threshold_bounded_by_min_win() {
        // At/after the threshold, admit exactly min_window_size (ceiling ample).
        assert_eq!(admit_bytes(0, PERSIST, PERSIST, 50_000, MIN_WIN, usize::MAX), MIN_WIN);
        assert_eq!(
            admit_bytes(0, PERSIST + 5, PERSIST, 50_000, MIN_WIN, usize::MAX),
            MIN_WIN
        );
    }

    #[test]
    fn persist_never_exceeds_surb_ceiling() {
        // Anti-grief: the probe is still capped by the SURB ceiling.
        // inflight 50_000, ceiling 52_000 ⇒ only 2_000 headroom even though min_window_size is 4_096.
        assert_eq!(admit_bytes(0, PERSIST, PERSIST, 50_000, MIN_WIN, 52_000), 2_000);
        // inflight already at/over the ceiling ⇒ probe admits nothing (never overspends SURBs).
        assert_eq!(admit_bytes(0, PERSIST, PERSIST, 50_000, MIN_WIN, 50_000), 0);
        assert_eq!(admit_bytes(0, PERSIST, PERSIST, 50_000, MIN_WIN, 40_000), 0);
    }

    #[test]
    fn surb_supply_ceiling_scales_with_buffer_level() {
        let supply = SurbSupply::new(balancer(7_000, 1_000), 1_000);
        assert_eq!(supply.max_admissible_inflight(), 1_000 * 1_000);
    }

    #[test]
    fn surb_supply_hard_backoff_when_empty() {
        let supply = SurbSupply::new(balancer(7_000, 0), 1_000);
        assert_eq!(supply.backoff_hint(), Some(Backoff::Hard));
    }

    #[test]
    fn surb_supply_soft_backoff_below_watermark() {
        // 10% of 7000 target = 700 < 25% watermark (1750) ⇒ Soft.
        let supply = SurbSupply::new(balancer(7_000, 700), 1_000);
        assert_eq!(supply.backoff_hint(), Some(Backoff::Soft));
    }

    #[test]
    fn surb_supply_healthy_gives_no_backoff() {
        // Healthy buffer must NOT be a "go faster" signal.
        let supply = SurbSupply::new(balancer(7_000, 6_000), 1_000);
        assert_eq!(supply.backoff_hint(), None);
    }

    #[test]
    fn surb_supply_disabled_balancer_has_no_ceiling() {
        let supply = SurbSupply::new(balancer(0, 0), 1_000);
        assert_eq!(supply.max_admissible_inflight(), usize::MAX);
        assert_eq!(supply.backoff_hint(), None);
    }

    // An in-memory duplex end used as the paced writer's inner: writes accumulate into a buffer.
    #[derive(Default)]
    struct Sink(Vec<u8>);
    impl futures::AsyncWrite for Sink {
        fn poll_write(mut self: Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            self.0.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn small_cfg() -> FlowControlConfig {
        FlowControlConfig {
            min_window_size: 1_000,
            max_window_size: 100_000,
            ai_step: 1_000,
            md_factor: 0.5,
            no_honest_deadline: Duration::from_millis(5),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn paced_writer_admits_up_to_the_floor_without_delivery() {
        let meter = DeliveryMeter::default();
        let clock = DeliveryClock::new(meter, None);
        let supply = SurbSupply::new(balancer(0, 0), 1_000); // no ceiling
        let mut w = PacedWriter::new(Sink::default(), small_cfg(), clock, supply);
        // The floor is 1000 bytes; a single write is capped to the admissible window.
        let n = w.write(&[7u8; 10_000]).await.unwrap();
        assert!(n <= 1_000, "first write cannot exceed the floor window, got {n}");
        assert!(n > 0);
    }

    #[tokio::test]
    async fn paced_writer_reopens_after_delivery() {
        let meter = DeliveryMeter::default();
        let clock = DeliveryClock::new(meter.clone(), None);
        let supply = SurbSupply::new(balancer(0, 0), 1_000);
        let mut w = PacedWriter::new(Sink::default(), small_cfg(), clock, supply);

        // Fill the floor window.
        let first = w.write(&[0u8; 10_000]).await.unwrap();
        assert!(first > 0);
        // Prove delivery of everything sent so far → window grows and admits more.
        meter.record_acked(first);
        let second = w.write(&[0u8; 10_000]).await.unwrap();
        assert!(second > 0, "delivery must reopen the window");
    }
}
