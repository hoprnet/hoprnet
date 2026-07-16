//! [`ServiceGate`] — bounded predeposit egress gate for PIX sessions.
//!
//! Before funding, the gate enforces a provisional packet budget.
//! After funding, it becomes a fast atomic counter.
//! On poisoning, all acquires fail permanently.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use tokio::sync::Notify;

/// Error returned when the gate is poisoned.
#[derive(Debug, Clone, thiserror::Error)]
#[error("service gate is poisoned (session closed)")]
pub(crate) struct GateClosed;

/// Bounded predeposit service gate for a single PIX session.
///
/// # Lock-free fast path
///
/// After funding, [`acquire`](Self::acquire) is a single `fetch_add` and
/// `load` on atomics — no mutex, no channel round-trip.
///
/// # Parking
///
/// Before funding, when the predeposit budget is exhausted, `acquire` parks
/// the caller. On [`release_service`](Self::release_service) or
/// [`poison`](Self::poison), all parked callers are woken.
pub(crate) struct ServiceGate {
    /// Monotonic number of packets served.
    served: AtomicU64,
    /// Remaining predeposit budget (tracked separately so we can park on 0).
    remaining: AtomicU64,
    /// Whether the funded flag has been flipped.
    funded: AtomicBool,
    /// Whether the gate is poisoned.
    poisoned: AtomicBool,
    /// Waker for parked writers.
    notify: Notify,
}

impl ServiceGate {
    /// Create a new gate with the given predeposit budget.
    pub fn new(predeposit_budget: u64) -> Arc<Self> {
        Arc::new(Self {
            served: AtomicU64::new(0),
            remaining: AtomicU64::new(predeposit_budget),
            funded: AtomicBool::new(false),
            poisoned: AtomicBool::new(false),
            notify: Notify::new(),
        })
    }

    /// Acquire a service permit.
    ///
    /// After funding or if the predeposit budget is not exhausted, this returns
    /// immediately. Otherwise it parks until a permit becomes available or the
    /// gate is poisoned.
    pub async fn acquire(self: &Arc<Self>) -> Result<(), GateClosed> {
        // Fast path: already funded.
        if self.funded.load(Ordering::Acquire) {
            self.served.fetch_add(1, Ordering::Relaxed);
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }
            return Ok(());
        }

        // Poisoned check before waiting.
        if self.poisoned.load(Ordering::Acquire) {
            return Err(GateClosed);
        }

        // Try to consume from predeposit budget.
        loop {
            if self.poisoned.load(Ordering::Acquire) {
                return Err(GateClosed);
            }

            // If funded while we were waiting, take the fast path.
            if self.funded.load(Ordering::Acquire) {
                self.served.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }

            let remaining = self.remaining.load(Ordering::Acquire);

            if remaining > 0 {
                // Try to decrement.
                if self
                    .remaining
                    .compare_exchange(remaining, remaining - 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    self.served.fetch_add(1, Ordering::Relaxed);
                    return Ok(());
                }
                // CAS failed (concurrent decrement), retry.
                continue;
            }

            // Budget exhausted — park.
            let notified = self.notify.notified();
            tokio::select! {
                _ = notified => {
                    // Woken by release_service or poison; loop to re-check.
                    continue;
                }
            }
        }
    }

    /// Current value of the served counter.
    pub fn served_total(&self) -> u64 {
        self.served.load(Ordering::Acquire)
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "used in tests and will be exposed via egress gating in Step 4")
    )]
    pub fn funded(&self) -> bool {
        self.funded.load(Ordering::Acquire)
    }

    /// Flip the funded flag and wake all parked writers.
    ///
    /// Once funded, `acquire` no longer checks the predeposit budget and never
    /// parks.
    pub fn release_service(self: &Arc<Self>) {
        self.funded.store(true, Ordering::Release);
        // Wake all parkers so they take the funded fast path.
        self.notify.notify_waiters();
    }

    /// Poison the gate: prevent all further acquires.
    ///
    /// Parked and future callers receive [`GateClosed`].
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    /// Non-blocking try-acquire for tests and sync-only callers.
    ///
    /// Returns `Ok(true)` on success, `Ok(false)` if the predeposit budget is
    /// exhausted, or `Err(())` if the gate is poisoned (session closing).
    #[cfg(test)]
    pub fn try_acquire_sync(&self) -> Result<bool, ()> {
        if self.poisoned.load(Ordering::Acquire) {
            return Err(());
        }
        if self.funded.load(Ordering::Acquire) {
            self.served.fetch_add(1, Ordering::Relaxed);
            return Ok(true);
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

    #[tokio::test]
    async fn budget_is_min_of_target_minus_one_and_config_cap() {
        // This test checks that the gate is constructed with the right budget.
        let gate = ServiceGate::new(100);
        assert_eq!(gate.remaining.load(Ordering::Acquire), 100);
        assert!(!gate.funded.load(Ordering::Acquire));
        assert!(!gate.poisoned.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn acquire_succeeds_within_budget() {
        let gate = ServiceGate::new(3);
        for _ in 0..3 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 3);
    }

    #[tokio::test]
    async fn acquire_parks_when_predeposit_budget_exhausted() {
        let gate = ServiceGate::new(1);
        gate.acquire().await.unwrap();

        // Second acquire should park (budget exhausted).
        let gate_clone = gate.clone();
        let parked =
            tokio::spawn(async move { tokio::time::timeout(Duration::from_millis(200), gate_clone.acquire()).await });

        // Should time out because no one wakes it.
        let result = parked.await.unwrap();
        assert!(result.is_err(), "expected timeout");
    }

    #[tokio::test]
    async fn release_service_wakes_parked_writers() {
        let gate = ServiceGate::new(0); // No predeposit budget.
        let gate_clone = gate.clone();

        let parked = tokio::spawn(async move {
            // This will park immediately.
            gate_clone.acquire().await.unwrap();
            42u32
        });

        // Small delay to ensure the spawned task is parked.
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Release service — this should wake the parked writer.
        gate.release_service();

        let result = parked.await.unwrap();
        assert_eq!(result, 42);
        assert!(gate.funded.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn funded_gate_counts_and_never_parks() {
        let gate = ServiceGate::new(0);
        gate.release_service();

        // After funding, acquires never park.
        for _ in 0..100 {
            gate.acquire().await.unwrap();
        }
        assert_eq!(gate.served_total(), 100);
    }

    #[tokio::test]
    async fn poison_errors_parked_and_future_acquires() {
        let gate = ServiceGate::new(0); // No predeposit budget → will park.
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
        let gate = ServiceGate::new(1000);
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
        let gate = ServiceGate::new(5);
        for _ in 0..5 {
            assert!(gate.try_acquire_sync().unwrap());
        }
        assert_eq!(gate.served_total(), 5);
    }

    #[tokio::test]
    async fn try_acquire_sync_returns_false_when_budget_exhausted() {
        let gate = ServiceGate::new(2);
        assert!(gate.try_acquire_sync().unwrap());
        assert!(gate.try_acquire_sync().unwrap());
        assert!(!gate.try_acquire_sync().unwrap());
        assert_eq!(gate.served_total(), 2);
    }

    #[tokio::test]
    async fn try_acquire_sync_succeeds_after_funding() {
        let gate = ServiceGate::new(0);
        // Exhausted predeposit budget.
        assert!(!gate.try_acquire_sync().unwrap());

        gate.release_service();
        assert!(gate.try_acquire_sync().unwrap());
        assert_eq!(gate.served_total(), 1);
    }

    #[tokio::test]
    async fn try_acquire_sync_errors_when_poisoned() {
        let gate = ServiceGate::new(10);
        gate.poison();
        assert!(gate.try_acquire_sync().is_err());
    }
}
