//! [`ServiceGate`] — bounded predeposit egress gate for PIX sessions.
//!
//! Before funding, the gate enforces a provisional packet budget from Exit to Entry.
//! After funding, it enforces a ceiling on packets served without SSA
//! recovery progress as a defense-in-depth backstop.
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
/// Before funding, when the predeposit budget is exhausted, [`acquire`](Self::acquire) parks
/// the caller. After funding, when the served-without-progress ceiling is exceeded, it
/// parks on the same mechanism.
/// On [`release_service`](Self::release_service), [`notify_progress`](Self::notify_progress),
/// or [`poison`](Self::poison), all parked callers are woken.
pub struct ServiceGate {
    /// Monotonic number of packets served.
    served: AtomicU64,
    /// Remaining predeposit budget (tracked separately so we can park on 0).
    remaining: AtomicU64,
    /// Whether the funded flag has been flipped.
    funded: AtomicBool,
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
            remaining: AtomicU64::new(predeposit_budget),
            funded: AtomicBool::new(false),
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
                    if !self.funded.load(Ordering::Acquire) {
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
            if self.funded.load(Ordering::Acquire) {
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

    /// Flip the funded flag and wake all parked writers.
    ///
    /// Once funded, `acquire` enforces the served-without-progress ceiling
    /// instead of the predeposit budget.
    ///
    /// Snapshots `served_total` into `served_at_last_progress` so the ceiling
    /// check starts from the moment of funding and does not count predeposit
    /// packets against the post-funding budget.
    pub fn release_service(self: &Arc<Self>) {
        self.funded.store(true, Ordering::Release);
        // Snapshot the served counter at the moment of funding so the ceiling
        // check does not count predeposit traffic against the post-funding
        // max_served_without_progress budget.
        self.served_at_last_progress
            .store(self.served.load(Ordering::Acquire), Ordering::Release);
        // Wake all parkers — predeposit-parked writers re-enter and take the
        // funded path, which checks the ceiling.
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

    /// Non-blocking try-acquire for tests and sync-only callers.
    ///
    /// Returns `Ok(true)` on success, `Ok(false)` if the predeposit budget is
    /// exhausted (and gate not yet funded) or the ceiling is exceeded (gate
    /// funded), or `Err(())` if poisoned.
    #[cfg(test)]
    pub fn try_acquire_sync(&self) -> Result<bool, ()> {
        if self.poisoned.load(Ordering::Acquire) {
            return Err(());
        }
        if self.funded.load(Ordering::Acquire) {
            let served = self.served.load(Ordering::Acquire);
            let base = self.served_at_last_progress.load(Ordering::Acquire);
            if served.saturating_sub(base) >= self.ceiling.load(Ordering::Acquire) {
                return Ok(false);
            }
            if self
                .served
                .compare_exchange(served, served + 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(true);
            }
            return Ok(false);
        }
        // Try to consume from predeposit budget (non-blocking CAS loop).
        loop {
            let remaining = self.remaining.load(Ordering::Acquire);
            if remaining == 0 {
                return Ok(false);
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

    #[tokio::test]
    async fn budget_is_min_of_target_minus_one_and_config_cap() {
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
}
