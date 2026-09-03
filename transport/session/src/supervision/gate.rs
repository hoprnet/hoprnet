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
    /// progress (see [`max_served_without_progress`](Self::ceiling)). Parks on
    /// [`SlotNotify`] when the ceiling or predeposit budget is exceeded.
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
                    self.served.fetch_add(1, Ordering::Relaxed);
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
    /// Returns `Ok(true)` on success, `Ok(false)` if the predeposit budget is
    /// exhausted (and gate not yet funded) or the ceiling is exceeded (gate
    /// funded), or [`GateClosed`] if poisoned.
    ///
    /// `Ok(false)` means *the gate refused*, and only ever that: both branches retry a lost
    /// compare-exchange rather than reporting it. Contention on `served` is not a refusal — service
    /// was available and the caller would be turned away anyway — and with several concurrent egress
    /// writers, reporting it as one converts contention into spurious refusals on the documented
    /// fast path.
    ///
    /// Every outgoing data packet of a supervised Session comes through here, and service is
    /// available for all but a vanishing fraction of them, so this answering synchronously is what
    /// keeps gating off the allocator: only [`acquire`](Self::acquire)'s parking path needs a future
    /// large enough to box, and that path is about to block anyway.
    pub fn try_acquire_sync(&self) -> Result<bool, GateClosed> {
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
                    return Ok(false);
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
                    return Ok(true);
                }
                continue;
            }

            // Try to consume from the current front's predeposit budget.
            let remaining = self.remaining.load(Ordering::Acquire);
            if remaining == 0 {
                if !self.mode_is_current(mode_epoch) {
                    continue;
                }
                return Ok(false);
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
                self.served.fetch_add(1, Ordering::Relaxed);
                return Ok(true);
            }
        }
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
        assert!(
            !gate.try_acquire_sync().expect("a fresh gate is not poisoned"),
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
        assert!(gate.try_acquire_sync().expect("a funded gate is not poisoned"));
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
            assert!(gate.try_acquire_sync().unwrap());
        }
        assert_eq!(gate.served_total(), 5);
    }

    #[tokio::test]
    async fn try_acquire_sync_returns_false_when_budget_exhausted() {
        let gate = gate_with_ceiling(2);
        assert!(gate.try_acquire_sync().unwrap());
        assert!(gate.try_acquire_sync().unwrap());
        assert!(!gate.try_acquire_sync().unwrap());
        assert_eq!(gate.served_total(), 2);
    }

    #[tokio::test]
    async fn try_acquire_sync_succeeds_after_funding() {
        let gate = gate_with_ceiling(0);

        assert!(!gate.try_acquire_sync().unwrap());
        gate.release_service();
        assert!(gate.try_acquire_sync().unwrap());
        assert_eq!(gate.served_total(), 1);
    }

    #[tokio::test]
    async fn try_acquire_sync_honors_ceiling_after_funding() {
        let gate = ServiceGate::new(0, 5);
        gate.release_service();

        for _ in 0..5 {
            assert!(gate.try_acquire_sync().unwrap());
        }
        // 6th should hit the ceiling.
        assert!(!gate.try_acquire_sync().unwrap());

        // Progress resets it.
        gate.notify_progress();
        assert!(gate.try_acquire_sync().unwrap());
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

        assert!(gate.try_acquire_sync()?);
        gate.release_service();
        assert!(gate.try_acquire_sync()?);

        gate.withhold_service();
        assert!(!gate.funded());
        assert!(gate.try_acquire_sync()?);
        assert!(gate.try_acquire_sync()?);
        assert!(!gate.try_acquire_sync()?);
        Ok(())
    }

    #[tokio::test]
    async fn strict_prepay_is_restored_when_service_is_withheld() -> anyhow::Result<()> {
        let gate = ServiceGate::new(0, 10);
        gate.release_service();
        assert!(gate.try_acquire_sync()?);

        gate.withhold_service();
        assert!(!gate.try_acquire_sync()?);
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
        assert!(gate.try_acquire_sync()?);

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

    #[tokio::test]
    async fn a_new_funded_front_gets_a_fresh_progress_ceiling() -> anyhow::Result<()> {
        let gate = ServiceGate::new(0, 2);
        gate.release_service();
        assert!(gate.try_acquire_sync()?);
        assert!(gate.try_acquire_sync()?);
        assert!(!gate.try_acquire_sync()?);

        // A funded-to-funded front handoff stays open but starts a new ceiling window.
        gate.release_service();
        assert!(gate.try_acquire_sync()?);
        assert!(gate.try_acquire_sync()?);
        assert!(!gate.try_acquire_sync()?);
        Ok(())
    }
}
