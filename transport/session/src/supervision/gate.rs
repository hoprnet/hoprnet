//! [`ServiceGate`] — bounded predeposit egress gate for PIX sessions.
//!
//! While the current front cycle is unfunded, the gate enforces a provisional packet budget from
//! Exit to Entry. Once that cycle is funded, it enforces a ceiling on packets served without its
//! recovery progress as a defense-in-depth backstop. A paid handoff restores the allowance for an
//! unfunded successor.
//! On poisoning, all acquires fail permanently.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::utils::SlotNotify;

/// Error returned when the gate is poisoned.
#[derive(Debug, Clone, thiserror::Error)]
#[error("service gate is poisoned (session closed)")]
pub struct GateClosed;

/// Why the gate refused a packet.
///
/// A closed enum, so it can be a metric label — the same argument as
/// `SessionPixCloseReason`, and it is carried for the same reason:
/// an operator asked "why is egress stalled" needs to tell an Entry that has not deposited from one
/// that has deposited but stopped returning shares. Those are different faults with different
/// remedies, and a bare "refused" conflates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum GateBlockReason {
    /// Unfunded, and the current front has spent its whole predeposit allowance. Service resumes
    /// when the deposit confirms, or when a paid handoff restores the allowance for a successor.
    PredepositExhausted,
    /// Funded, but service has run `max_served_without_progress` packets ahead of the shares coming
    /// back. Service resumes on the next validated progress notification for the front cycle.
    ShareLag,
}

/// What the gate answered a packet with.
///
/// Replaces the `bool` this used to be. Both refusal sites already knew which case they were —
/// carrying it costs nothing, allocates nothing, and is what lets the egress path report *why* it
/// parked rather than only that it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    /// A permit was taken; the packet may go.
    Admitted,
    /// No permit is available right now, for this reason.
    Blocked(GateBlockReason),
}

/// Bounded predeposit service gate for a single PIX session.
///
/// # Parking
///
/// While the front is unfunded, exhausting its predeposit budget parks [`acquire`](Self::acquire).
/// Once funded, exceeding the served-without-progress ceiling parks on the same mechanism. On
/// [`release_service`](Self::release_service), [`withhold_service`](Self::withhold_service),
/// [`notify_progress`](Self::notify_progress), or [`poison`](Self::poison), all parked callers are
/// woken.
pub struct ServiceGate {
    /// Monotonic number of packets served.
    served: AtomicU64,
    /// Of those, the ones taken from a predeposit allowance rather than from funded service.
    ///
    /// Monotonic over the life of the gate and never reset by a rotation, so it is a share of
    /// [`served`](Self::served) rather than a per-front figure — see
    /// [`served_split`](Self::served_split), which is the only reader.
    served_predeposit: AtomicU64,
    /// Predeposit budget restored after each paid front-cycle handoff.
    predeposit_budget: u64,
    /// Remaining predeposit budget (tracked separately so we can park on 0).
    remaining: AtomicU64,
    /// Whether the current front cycle is funded.
    funded: AtomicBool,
    /// Incremented after every mode publication.
    ///
    /// Permit acquisition performs an RMW on this epoch before committing its counter update. That
    /// gives mode transitions and permits one atomic ordering point: a permit either belongs to the
    /// old front or observes the new mode, rather than loading `funded` before a transition and
    /// committing after it.
    mode_epoch: AtomicU64,
    /// Whether the gate is poisoned.
    poisoned: AtomicBool,
    /// Waker for parked writers.
    notify: SlotNotify,
    /// Ceiling on packets served since last progress notification.
    ceiling: AtomicU64,
    /// Snapshot of `served` at last progress notification.
    served_at_last_progress: AtomicU64,
}

impl ServiceGate {
    /// Create a new gate with the given predeposit budget and progress ceiling.
    pub fn new(predeposit_budget: u64, max_served_without_progress: u64) -> Arc<Self> {
        Arc::new(Self {
            served: AtomicU64::new(0),
            served_predeposit: AtomicU64::new(0),
            predeposit_budget,
            remaining: AtomicU64::new(predeposit_budget),
            funded: AtomicBool::new(false),
            mode_epoch: AtomicU64::new(0),
            poisoned: AtomicBool::new(false),
            notify: SlotNotify::new(),
            ceiling: AtomicU64::new(max_served_without_progress),
            served_at_last_progress: AtomicU64::new(0),
        })
    }

    /// Acquire a service permit.
    ///
    /// After funding, enforces a ceiling on packets served without SSA recovery
    /// progress (see the `ceiling` field). Parks on `SlotNotify` when the ceiling or predeposit
    /// budget is exceeded.
    pub async fn acquire(self: &Arc<Self>) -> Result<(), GateClosed> {
        if self.poisoned.load(Ordering::Acquire) {
            return Err(GateClosed);
        }

        loop {
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }

            let mode_epoch = self.mode_epoch.load(Ordering::Acquire);
            if self.funded.load(Ordering::Acquire) {
                // Funded path: ceiling-checking CAS loop.
                let served = self.served.load(Ordering::Acquire);
                let base = self.served_at_last_progress.load(Ordering::Acquire);
                if served.saturating_sub(base) >= self.ceiling.load(Ordering::Acquire) {
                    // Ceiling exceeded — park.
                    let notified = self.notify.notified();

                    // Double-check after registering interest.
                    if self.poisoned.load(Ordering::Acquire) {
                        return Err(GateClosed);
                    }
                    if self.mode_epoch.load(Ordering::Acquire) != mode_epoch || !self.funded.load(Ordering::Acquire) {
                        continue;
                    }
                    let served2 = self.served.load(Ordering::Acquire);
                    let base2 = self.served_at_last_progress.load(Ordering::Acquire);
                    if served2.saturating_sub(base2) < self.ceiling.load(Ordering::Acquire) {
                        // Progress happened while registering — retry.
                        continue;
                    }

                    notified.await;
                    continue;
                }

                // Re-check poison right before CAS so that a concurrent
                // poison() is not missed between the entry check and here.
                if self.poisoned.load(Ordering::Acquire) {
                    return Err(GateClosed);
                }

                if !self.mode_is_current(mode_epoch) {
                    continue;
                }

                if self
                    .served
                    .compare_exchange(served, served + 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return Ok(());
                }
                // CAS failed — retry.
                continue;
            }

            // Not yet funded — try predeposit budget.
            let remaining = self.remaining.load(Ordering::Acquire);

            if remaining > 0 {
                // Re-check poison right before CAS so that a concurrent
                // poison() is not missed between the entry check and here.
                if self.poisoned.load(Ordering::Acquire) {
                    return Err(GateClosed);
                }
                if !self.mode_is_current(mode_epoch) {
                    continue;
                }
                if self
                    .remaining
                    .compare_exchange(remaining, remaining - 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    // Counted here as well as on the synchronous path: a writer that parked on an
                    // exhausted allowance and was woken by a paid handoff takes its permit through
                    // *this* branch, and a split that missed those would attribute predeposit
                    // service to the funded bucket.
                    //
                    // Published before `served`, and `served` published with `Release`, so that
                    // `served_split`'s acquire load of `served` cannot see this permit in the total
                    // while still reading the old predeposit count — see its documentation.
                    self.served_predeposit.fetch_add(1, Ordering::Relaxed);
                    self.served.fetch_add(1, Ordering::Release);
                    return Ok(());
                }
                // CAS failed — retry.
                continue;
            }

            // Budget exhausted — park.
            //
            // Register interest FIRST, then re-check conditions. This
            // prevents a missed wake-up: without the double-check, a
            // concurrent release_service()/poison() can call
            // notify_waiters() between the budget check above and the
            // Notified creation below, and the new Notified would never
            // observe that notification.
            let notified = self.notify.notified();

            // Double-check: after registering, re-read all conditions
            // that could have changed since the last load above.
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }
            if self.mode_epoch.load(Ordering::Acquire) != mode_epoch || self.funded.load(Ordering::Acquire) {
                // Re-enter the loop — the funded path handles ceiling checks.
                continue;
            }
            if self.remaining.load(Ordering::Acquire) > 0 {
                continue;
            }

            // Budget exhausted — park and wait for wake-up.
            notified.await;
        }
    }

    /// Current value of the served counter.
    pub fn served_total(&self) -> u64 {
        self.served.load(Ordering::Acquire)
    }

    /// Packets served against the current front's predeposit allowance and not yet paid for.
    ///
    /// Zero while funded, and that is the definition rather than an approximation: funding is
    /// exactly the event that converts the allowance the current front spent into service the Entry
    /// has paid for, so there is nothing outstanding to report. A later
    /// [`withhold_service`](Self::withhold_service) restores the whole allowance for an unfunded
    /// successor, and the figure starts climbing again from zero for that cycle.
    ///
    /// Read once per supervisor turn rather than per packet, so the two relaxed loads are not on any
    /// hot path.
    pub fn predeposit_exposure(&self) -> u64 {
        if self.funded.load(Ordering::Acquire) {
            return 0;
        }
        self.predeposit_budget
            .saturating_sub(self.remaining.load(Ordering::Acquire))
    }

    #[cfg(test)]
    pub fn funded(&self) -> bool {
        self.funded.load(Ordering::Acquire)
    }

    /// Linearization point shared by a permit and mode publication.
    fn mode_is_current(&self, epoch: u64) -> bool {
        self.mode_epoch
            .compare_exchange(epoch, epoch, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Publish funded mode for the current front and wake all parked writers.
    ///
    /// `acquire` then enforces the served-without-progress ceiling instead of the predeposit budget.
    /// Calling this while already funded represents a funded-to-funded front handoff and starts the
    /// new front with a fresh ceiling window.
    ///
    /// Snapshots `served_total` into `served_at_last_progress` so the ceiling
    /// check starts from the moment of funding and does not count predeposit
    /// packets against the post-funding budget.
    pub fn release_service(self: &Arc<Self>) {
        // Snapshot the served counter at the moment of funding so the ceiling
        // check does not count predeposit traffic against the post-funding
        // max_served_without_progress budget.
        self.served_at_last_progress
            .store(self.served.load(Ordering::Acquire), Ordering::Release);
        // Published *after* the watermark, so a caller that observes `funded` observes the snapshot
        // too — the release store orders everything sequenced before it. The other way round leaves
        // a window in which the funded branch judges the whole predeposit-era `served` count against
        // the ceiling and refuses service that is in fact available. The window is self-clearing,
        // since the wake below releases whoever parked in it, but it costs a spurious refusal on
        // every funding event for nothing.
        self.funded.store(true, Ordering::Release);
        self.mode_epoch.fetch_add(1, Ordering::Release);
        // Wake all parkers — predeposit-parked writers re-enter and take the
        // funded path, which checks the ceiling.
        self.notify.notify_waiters();
    }

    /// Return to predeposit mode for the next unfunded front cycle.
    ///
    /// The allowance is restored before the mode is published, so a permit that observes the new
    /// epoch also observes the complete budget. Parked ceiling waiters are woken to re-evaluate the
    /// predeposit branch; with a zero allowance they correctly park again.
    pub fn withhold_service(self: &Arc<Self>) {
        self.remaining.store(self.predeposit_budget, Ordering::Release);
        self.funded.store(false, Ordering::Release);
        self.mode_epoch.fetch_add(1, Ordering::Release);
        self.notify.notify_waiters();
    }

    /// Record SSA recovery progress: snapshots the served counter so the
    /// ceiling reopens, and wakes any writers parked on the ceiling.
    pub fn notify_progress(self: &Arc<Self>) {
        self.served_at_last_progress
            .store(self.served.load(Ordering::Acquire), Ordering::Release);
        self.notify.notify_waiters();
    }

    /// Poison the gate: prevent all further acquires.
    ///
    /// Parked and future callers receive [`GateClosed`].
    ///
    /// # Semantics
    ///
    /// After `poison()` returns, at most one in-flight `acquire()` per
    /// concurrent caller may still observe the gate as not poisoned (a
    /// parked awaiter that wakes before the poison store is visible).
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    /// Non-blocking try-acquire — the egress fast path.
    ///
    /// Returns [`GateVerdict::Admitted`] on success, [`GateVerdict::Blocked`] with the reason if the
    /// predeposit budget is exhausted (and the gate not yet funded) or the ceiling is exceeded (gate
    /// funded), or [`GateClosed`] if poisoned.
    ///
    /// `Blocked` means *the gate refused*, and only ever that: both branches retry a lost
    /// compare-exchange rather than reporting it. Contention on `served` is not a refusal — service
    /// was available and the caller would be turned away anyway — and with several concurrent egress
    /// writers, reporting it as one converts contention into spurious refusals on the documented
    /// fast path.
    ///
    /// Every outgoing data packet of a supervised Session comes through here, and service is
    /// available for all but a vanishing fraction of them, so this answering synchronously is what
    /// keeps gating off the allocator: only [`acquire`](Self::acquire)'s parking path needs a future
    /// large enough to box, and that path is about to block anyway. The same argument is why the
    /// funded branch below does not count anything of its own: it is the steady state, at the
    /// Session's full packet rate, and the split it would produce is available by subtraction.
    pub fn try_acquire_sync(&self) -> Result<GateVerdict, GateClosed> {
        loop {
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }

            let mode_epoch = self.mode_epoch.load(Ordering::Acquire);
            if self.funded.load(Ordering::Acquire) {
                let served = self.served.load(Ordering::Acquire);
                let base = self.served_at_last_progress.load(Ordering::Acquire);
                if served.saturating_sub(base) >= self.ceiling.load(Ordering::Acquire) {
                    if !self.mode_is_current(mode_epoch) {
                        continue;
                    }
                    return Ok(GateVerdict::Blocked(GateBlockReason::ShareLag));
                }
                if self.poisoned.load(Ordering::Acquire) {
                    return Err(GateClosed);
                }
                if !self.mode_is_current(mode_epoch) {
                    continue;
                }
                if self
                    .served
                    .compare_exchange(served, served + 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return Ok(GateVerdict::Admitted);
                }
                continue;
            }

            // Try to consume from the current front's predeposit budget.
            let remaining = self.remaining.load(Ordering::Acquire);
            if remaining == 0 {
                if !self.mode_is_current(mode_epoch) {
                    continue;
                }
                return Ok(GateVerdict::Blocked(GateBlockReason::PredepositExhausted));
            }
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }
            if !self.mode_is_current(mode_epoch) {
                continue;
            }
            if self
                .remaining
                .compare_exchange(remaining, remaining - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                // Predeposit first, then `served` with `Release`: the publication order
                // `served_split` relies on, documented there.
                self.served_predeposit.fetch_add(1, Ordering::Relaxed);
                self.served.fetch_add(1, Ordering::Release);
                return Ok(GateVerdict::Admitted);
            }
        }
    }

    /// Packets this gate has admitted, split into `(predeposit, funded)`.
    ///
    /// Counted on the predeposit branch only, and derived for the funded one. That asymmetry is the
    /// whole point: at deployed dimensions the funded branch carries essentially all of a Session's
    /// egress — tens of thousands of packets a second across a node — while the predeposit branch
    /// carries at most `max_predeposit_packets` per front rotation. A second `fetch_add` on the
    /// steady-state path would be paid on every packet to produce a number that subtraction already
    /// gives exactly.
    ///
    /// # The publication order, and why it is that way round
    ///
    /// The two loads are not atomic with respect to each other, so a concurrent predeposit permit
    /// can land between them and be seen by one load but not the other. Which of the two tears is
    /// possible decides whether that permit is merely *late* or is actually *miscounted*, and the
    /// writers pick which by the order they publish in.
    ///
    /// A predeposit permit therefore increments `served_predeposit` first and publishes `served`
    /// with [`Release`](Ordering::Release). The acquire load of `served` below synchronizes with
    /// that store, so seeing the permit in the total guarantees seeing it in the predeposit count
    /// too: the derived funded figure can never come out one *high*, which would report a
    /// predeposit packet as funded and then report it again as predeposit on the next read.
    ///
    /// The surviving tear is the harmless one — `served_predeposit` new against `served` old, which
    /// makes the derived funded figure one *low*. Nothing is lost and the next read is consistent
    /// again, but a consumer keeping a watermark must not store the regressed value or it will
    /// re-cross that ground: see
    /// [`PixSessionTelemetry::flush_egress`](super::telemetry::PixSessionTelemetry), which clamps.
    pub fn served_split(&self) -> (u64, u64) {
        let served = self.served.load(Ordering::Acquire);
        let predeposit = self.served_predeposit.load(Ordering::Acquire);
        (predeposit, served.saturating_sub(predeposit))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// Helper: create a gate with a generous ceiling for tests that don't
    /// care about the ceiling behavior.
    fn gate_with_ceiling(predeposit: u64) -> Arc<ServiceGate> {
        ServiceGate::new(predeposit, u64::MAX)
    }

    /// `new` stores what it is given and starts both flags clear.
    ///
    /// Named after that and nothing more. The `min(target_useful_shares - 1, max_predeposit_packets)`
    /// this used to claim to test is computed in `spawn_supervisor_worker`, not here — this file
    /// cannot observe it — and is covered there by `zero_predeposit_config_reaches_the_gate_as_strict_prepay`
    /// and `predeposit_budget_is_bounded_by_the_ssa_dimensions`.
    #[tokio::test]
    async fn new_stores_its_budget_and_ceiling_and_starts_unfunded() {
        let gate = ServiceGate::new(100, 256);
        assert_eq!(gate.remaining.load(Ordering::Acquire), 100);
        assert_eq!(gate.ceiling.load(Ordering::Acquire), 256);
        assert!(!gate.funded.load(Ordering::Acquire));
        assert!(!gate.poisoned.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn acquire_succeeds_within_budget() {
        let gate = gate_with_ceiling(3);
        for _ in 0..3 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 3);
    }

    #[tokio::test]
    async fn acquire_parks_when_predeposit_budget_exhausted() {
        let gate = gate_with_ceiling(1);
        gate.acquire().await.unwrap();

        let gate_clone = gate.clone();
        let parked =
            tokio::spawn(async move { tokio::time::timeout(Duration::from_millis(200), gate_clone.acquire()).await });

        let result = parked.await.unwrap();
        assert!(result.is_err(), "expected timeout");
    }

    #[tokio::test]
    async fn release_service_wakes_parked_writers() {
        let gate = gate_with_ceiling(0); // No predeposit budget.
        let gate_clone = gate.clone();

        let parked = tokio::spawn(async move {
            gate_clone.acquire().await.unwrap();
            42u32
        });

        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.release_service();

        let result = parked.await.unwrap();
        assert_eq!(result, 42);
        assert!(gate.funded.load(Ordering::Acquire));
    }

    /// Strict prepay: a gate with no predeposit budget serves nothing until it is funded.
    ///
    /// Several tests above already happen to use a zero budget to reach some other behaviour, but
    /// none of them states this configuration's contract, which is the whole of what an Exit
    /// choosing `max_predeposit_packets = 0` is buying. A change that admitted even one packet
    /// before funding would pass every one of those and fail only here.
    #[tokio::test]
    async fn a_zero_budget_gate_serves_nothing_until_funded() {
        let gate = ServiceGate::new(0, 10);

        // Nothing, on either path.
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync().expect("a fresh gate is not poisoned"),
            "the synchronous path must refuse while unfunded with no budget"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), gate.acquire())
                .await
                .is_err(),
            "the async path must park rather than admit"
        );
        assert_eq!(gate.served_total(), 0, "a refused packet must not be counted as served");

        // A writer parked before funding is woken by it, not left pending.
        let parked = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.acquire().await })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.release_service();
        parked
            .await
            .unwrap()
            .expect("funding must wake the writer parked on an empty budget");

        // Service is then ordinary, and the ceiling counts from funding rather than from a
        // predeposit allowance that was never spent.
        assert_eq!(
            GateVerdict::Admitted,
            gate.try_acquire_sync().expect("a funded gate is not poisoned")
        );
        assert_eq!(gate.served_total(), 2);
    }

    #[tokio::test]
    async fn funded_gate_surrenders_at_ceiling() {
        let gate = ServiceGate::new(0, 10); // Ceiling of 10.
        gate.release_service();

        // Serve up to the ceiling.
        for _ in 0..10 {
            gate.acquire().await.unwrap();
        }

        // 11th should park (ceiling exceeded).
        let gate_clone = gate.clone();
        let parked =
            tokio::spawn(async move { tokio::time::timeout(Duration::from_millis(100), gate_clone.acquire()).await });
        let result = parked.await.unwrap();
        assert!(result.is_err(), "expected timeout due to ceiling");

        assert_eq!(gate.served_total(), 10);
    }

    #[tokio::test]
    async fn notify_progress_resets_ceiling() {
        let gate = ServiceGate::new(0, 10);
        gate.release_service();

        for _ in 0..10 {
            gate.acquire().await.unwrap();
        }

        // Progress resets the ceiling.
        gate.notify_progress();

        // Now serve another 10.
        for _ in 0..10 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 20);
    }

    #[tokio::test]
    async fn notify_progress_wakes_ceiling_parked_writer() {
        let gate = ServiceGate::new(0, 5);
        gate.release_service();

        for _ in 0..5 {
            gate.acquire().await.unwrap();
        }

        let gate_clone = gate.clone();
        let parked = tokio::spawn(async move {
            gate_clone.acquire().await.unwrap();
            42u32
        });

        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.notify_progress();

        let result = parked.await.unwrap();
        assert_eq!(result, 42);
        assert_eq!(gate.served_total(), 6);
    }

    #[tokio::test]
    async fn poison_errors_parked_and_future_acquires() {
        let gate = gate_with_ceiling(0); // No predeposit budget → will park.
        let gate_clone = gate.clone();

        let parked = tokio::spawn(async move { gate_clone.acquire().await });

        tokio::time::sleep(Duration::from_millis(20)).await;
        gate.poison();

        let result = parked.await.unwrap();
        assert!(result.is_err());

        // Future acquires also fail.
        let gate_clone = gate.clone();
        let result = gate_clone.acquire().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn served_total_is_monotonic_under_concurrency() {
        let gate = gate_with_ceiling(1000);
        let mut handles = Vec::new();

        for _ in 0..10 {
            let g = gate.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    g.acquire().await.unwrap();
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(gate.served_total(), 1000);
    }

    #[tokio::test]
    async fn try_acquire_sync_succeeds_within_budget() {
        let gate = gate_with_ceiling(5);
        for _ in 0..5 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
        }
        assert_eq!(gate.served_total(), 5);
    }

    #[tokio::test]
    async fn try_acquire_sync_reports_predeposit_exhaustion_when_the_budget_is_spent() {
        let gate = gate_with_ceiling(2);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync().unwrap()
        );
        assert_eq!(gate.served_total(), 2);
    }

    #[tokio::test]
    async fn try_acquire_sync_succeeds_after_funding() {
        let gate = gate_with_ceiling(0);

        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync().unwrap()
        );
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
        assert_eq!(gate.served_total(), 1);
    }

    #[tokio::test]
    async fn try_acquire_sync_honors_ceiling_after_funding() {
        let gate = ServiceGate::new(0, 5);
        gate.release_service();

        for _ in 0..5 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
        }
        // 6th should hit the ceiling.
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::ShareLag),
            gate.try_acquire_sync().unwrap()
        );

        // Progress resets it.
        gate.notify_progress();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync().unwrap());
    }

    #[tokio::test]
    async fn try_acquire_sync_errors_when_poisoned() {
        let gate = gate_with_ceiling(10);
        gate.poison();
        assert!(gate.try_acquire_sync().is_err());
    }

    #[tokio::test]
    async fn ceiling_check_uses_saturating_sub_from_watermark() {
        // Pre-serve some packets via predeposit, then fund and check ceiling
        // starts fresh from the watermark, not from 0.
        let gate = ServiceGate::new(50, 10);
        for _ in 0..30 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 30);

        gate.release_service();
        gate.notify_progress(); // Watermark = 30, ceiling = 10.
        assert_eq!(gate.served_at_last_progress.load(Ordering::Acquire), 30);

        for _ in 0..10 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 40);

        // 41st should hit ceiling (40 - 30 >= 10).
        let gate_clone = gate.clone();
        let parked =
            tokio::spawn(async move { tokio::time::timeout(Duration::from_millis(50), gate_clone.acquire()).await });
        let result = parked.await.unwrap();
        assert!(result.is_err(), "expected timeout due to ceiling");
    }

    // -------------------------------------------------------------------
    // M-05: Funding watermark
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn release_service_snapshots_served_total_as_watermark() {
        let gate = ServiceGate::new(50, 5);
        for _ in 0..10 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 10);

        // release_service must snapshot served_total into served_at_last_progress.
        gate.release_service();
        assert_eq!(gate.served_at_last_progress.load(Ordering::Acquire), 10);
    }

    #[tokio::test]
    async fn release_service_after_ceiling_predeposit_unblocks_waiter() {
        // predeposit = 100, ceiling = 10.
        let gate = ServiceGate::new(100, 10);

        // Consume 30 predeposit packets — more than the ceiling.
        for _ in 0..30 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 30);

        // Funding snapshots served=30 into the watermark. The ceiling check
        // then sees 30 - 30 = 0 < 10, so the waiter is unblocked.
        gate.release_service();
        gate.acquire().await.unwrap();
        assert_eq!(gate.served_total(), 31);
    }

    #[tokio::test]
    async fn withholding_restores_the_predeposit_budget_for_the_next_paid_handoff() -> anyhow::Result<()> {
        let gate = ServiceGate::new(2, 10);

        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);

        gate.withhold_service();
        assert!(!gate.funded());
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync()?
        );
        Ok(())
    }

    #[tokio::test]
    async fn strict_prepay_is_restored_when_service_is_withheld() -> anyhow::Result<()> {
        let gate = ServiceGate::new(0, 10);
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);

        gate.withhold_service();
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync()?
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(50), gate.acquire())
                .await
                .is_err(),
            "the async path must park after a strict-prepay rotation too"
        );
        Ok(())
    }

    #[tokio::test]
    async fn withholding_wakes_a_ceiling_parked_writer_into_the_new_allowance() -> anyhow::Result<()> {
        let gate = ServiceGate::new(1, 1);
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);

        let mut parked = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.acquire().await })
        };
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut parked)
                .await
                .is_err(),
            "the writer must first park on the funded ceiling"
        );

        gate.withhold_service();
        tokio::time::timeout(Duration::from_secs(1), parked).await???;
        assert_eq!(gate.served_total(), 2);
        Ok(())
    }

    /// Exposure tracks the current front's unpaid service, and funding is what clears it.
    ///
    /// Zero once funded is the contract rather than an approximation: funding converts the
    /// allowance the front spent into service the Entry has paid for, so an Exit reading
    /// `hopr_pix_predeposit_exposure_packets` sees only what it is still owed.
    #[tokio::test]
    async fn predeposit_exposure_tracks_the_current_front_and_clears_on_funding() -> anyhow::Result<()> {
        let gate = ServiceGate::new(4, 10);
        assert_eq!(0, gate.predeposit_exposure(), "nothing served, nothing exposed");

        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(2, gate.predeposit_exposure());

        gate.release_service();
        assert_eq!(
            0,
            gate.predeposit_exposure(),
            "the deposit paid for what the allowance advanced"
        );

        // Funded service is not exposure however much of it there is.
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(0, gate.predeposit_exposure());

        // A paid handoff restores the whole allowance for an unfunded successor, and the successor's
        // own exposure starts again from nothing rather than inheriting its predecessor's.
        gate.withhold_service();
        assert_eq!(0, gate.predeposit_exposure());
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(1, gate.predeposit_exposure());
        Ok(())
    }

    /// The served split attributes each packet to whatever paid for it, across a full rotation.
    ///
    /// The funded component is derived rather than counted, so the property worth stating is that
    /// the derivation is exact: the two halves must always sum to `served_total`, whichever branch
    /// admitted the packet and whichever of the two entry points it came through.
    #[tokio::test]
    async fn the_served_split_attributes_every_packet_to_what_paid_for_it() -> anyhow::Result<()> {
        let gate = ServiceGate::new(3, 100);
        assert_eq!((0, 0), gate.served_split());

        // Two on the allowance, synchronously.
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!((2, 0), gate.served_split());

        // One more on the allowance, through the async path — which is the one a writer woken by a
        // handoff takes, and would be attributed to the wrong bucket if only the sync path counted.
        gate.acquire().await?;
        assert_eq!((3, 0), gate.served_split());

        // Funding moves the accounting without rewriting history: the three already served stay
        // charged to the allowance.
        gate.release_service();
        for _ in 0..5 {
            assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        }
        assert_eq!((3, 5), gate.served_split());

        // A paid handoff restores the allowance, and the next packets are unpaid again.
        gate.withhold_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!((4, 5), gate.served_split());

        let (predeposit, funded) = gate.served_split();
        assert_eq!(
            gate.served_total(),
            predeposit + funded,
            "the split must account for every packet the gate admitted"
        );
        Ok(())
    }

    /// A concurrent reader of the split never sees predeposit service as funded.
    ///
    /// The two loads in `served_split` are not atomic with respect to each other, so a permit
    /// landing between them is visible to one and not the other. On a gate that has never been
    /// funded the answer is knowable regardless: every packet came off the allowance, so the derived
    /// funded figure must be zero at *every* observation, however the reads interleave.
    ///
    /// That is the invariant the predeposit path's publication order buys — `served_predeposit`
    /// first, `served` released — and it is worth a test because the failure is silent: a reader
    /// that saw the total move before the split would emit the packet as funded, and the watermark
    /// on the other side cannot take that back.
    ///
    /// Note this cannot fail on x86, whose store ordering makes the wrong order accidentally
    /// correct. It is the weaker targets and the compiler's own freedom to reorder two relaxed
    /// read-modify-writes that this pins down.
    #[tokio::test]
    async fn a_concurrent_split_never_attributes_predeposit_service_to_the_funded_bucket() {
        const PACKETS: u64 = 20_000;

        let gate = ServiceGate::new(PACKETS, u64::MAX);
        // Both sides are released together. Without it the admissions can be over before the
        // sampling loop first looks, which would leave the test passing on nothing.
        let start = Arc::new(std::sync::Barrier::new(2));
        let admitting = {
            let gate = gate.clone();
            let start = start.clone();
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..PACKETS {
                    assert_eq!(
                        GateVerdict::Admitted,
                        gate.try_acquire_sync().expect("an unpoisoned gate with allowance left")
                    );
                }
            })
        };
        start.wait();

        // Sample first, check for completion second, so the body runs whatever the scheduler does.
        loop {
            let (predeposit, funded) = gate.served_split();
            assert_eq!(
                0, funded,
                "this gate was never funded, so every one of its {predeposit} packets came off the allowance — a \
                 non-zero funded figure is a torn read being counted"
            );
            if admitting.is_finished() {
                break;
            }
        }
        admitting.join().expect("the admitting thread must not panic");

        assert_eq!((PACKETS, 0), gate.served_split(), "and the final split is exact");
    }

    /// A strict-prepay gate has no allowance to expose, whatever happens to it.
    #[tokio::test]
    async fn a_zero_budget_gate_exposes_nothing() -> anyhow::Result<()> {
        let gate = ServiceGate::new(0, 10);
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::PredepositExhausted),
            gate.try_acquire_sync()?
        );
        assert_eq!(0, gate.predeposit_exposure());

        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(0, gate.predeposit_exposure());
        Ok(())
    }

    #[tokio::test]
    async fn a_new_funded_front_gets_a_fresh_progress_ceiling() -> anyhow::Result<()> {
        let gate = ServiceGate::new(0, 2);
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::ShareLag),
            gate.try_acquire_sync()?
        );

        // A funded-to-funded front handoff stays open but starts a new ceiling window.
        gate.release_service();
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(GateVerdict::Admitted, gate.try_acquire_sync()?);
        assert_eq!(
            GateVerdict::Blocked(GateBlockReason::ShareLag),
            gate.try_acquire_sync()?
        );
        Ok(())
    }
}
