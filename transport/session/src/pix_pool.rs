//! A thread pool of its own for the CPU-heavy PIX commitment work.
//!
//! On the shared CPU pool, the jobs of one commitment burst took every thread, and packets queued
//! behind them until the burst was done (hoprnet#8765).

use std::{sync::LazyLock, time::Instant};

use futures::channel::oneshot;
use hopr_utils::parallelize::cpu::{self, rayon};

/// PIX commitment work run by [`spawn_pix_blocking`], as `hopr_pix_pool_execution_seconds` labels it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum PixPoolOperation {
    /// An Exit decoding and storing the commitments of one `SsaCommit` message.
    CommitmentIngest,
    /// An Entry generating the polynomials and commitments of a new SSA cycle.
    CommitmentGeneration,
}

/// Half as many threads as the shared CPU pool, and at least one.
static PIX_POOL: LazyLock<Result<rayon::ThreadPool, rayon::ThreadPoolBuildError>> = LazyLock::new(|| {
    let shared_threads = match cpu::pool_thread_count() {
        0 => std::thread::available_parallelism().map_or(1, |n| n.get() / 2),
        n => n,
    };
    rayon::ThreadPoolBuilder::new()
        .num_threads((shared_threads / 2).max(1))
        .thread_name(|i| format!("pix-{i}"))
        .build()
});

/// Runs `f` on the PIX pool and returns its result, re-raising a panic of `f` in the caller.
///
/// Parallel iterators inside `f` stay on the PIX pool. The job is skipped if the returned future is
/// dropped before it starts.
pub(crate) async fn spawn_pix_blocking<R: Send + 'static>(
    operation: PixPoolOperation,
    f: impl FnOnce() -> R + Send + 'static,
) -> anyhow::Result<R> {
    let pool = PIX_POOL
        .as_ref()
        .map_err(|error| anyhow::anyhow!("the PIX thread pool could not be built: {error}"))?;

    let (tx, rx) = oneshot::channel();
    let span = tracing::Span::current();
    let submitted_at = Instant::now();
    pool.spawn_fifo(move || {
        if tx.is_canceled() {
            return;
        }
        let _span = span.entered();
        let queue_wait = submitted_at.elapsed();
        let started_at = Instant::now();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        #[cfg(feature = "telemetry")]
        crate::telemetry::pix::record_pix_pool_job(operation, queue_wait, started_at.elapsed());
        #[cfg(not(feature = "telemetry"))]
        let _ = (operation, queue_wait, started_at);
        let _ = tx.send(result);
    });

    match rx.await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(oneshot::Canceled) => Err(anyhow::anyhow!("a PIX pool job ended without a result")),
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;

    use super::*;

    #[tokio::test]
    async fn a_pix_job_returns_its_result() -> anyhow::Result<()> {
        let result = spawn_pix_blocking(PixPoolOperation::CommitmentIngest, || 42).await?;
        assert_eq!(result, 42);
        Ok(())
    }

    #[tokio::test]
    async fn parallel_work_inside_a_pix_job_stays_on_the_pix_pool() -> anyhow::Result<()> {
        use rayon::prelude::*;

        let thread_names = spawn_pix_blocking(PixPoolOperation::CommitmentGeneration, || {
            (0..256)
                .into_par_iter()
                .map(|_| std::thread::current().name().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .await?;

        assert!(
            thread_names
                .iter()
                .all(|name| name.as_deref().is_some_and(|name| name.starts_with("pix-"))),
            "every item must run on a PIX pool thread: {thread_names:?}"
        );
        Ok(())
    }

    #[cfg(panic = "unwind")]
    #[tokio::test]
    async fn a_panic_in_a_pix_job_reaches_the_caller() {
        let outcome = std::panic::AssertUnwindSafe(spawn_pix_blocking(PixPoolOperation::CommitmentIngest, || {
            panic!("pix job panic")
        }))
        .catch_unwind()
        .await;
        assert!(outcome.is_err(), "the panic must be re-raised in the caller");
    }
}
