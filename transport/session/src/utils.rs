/// Convenience function to copy data in both directions between a [`Session`] and arbitrary
/// async IO stream.
/// This function is only available with Tokio and will panic with other runtimes.
///
/// The `abort_stream` will terminate the transfer from the `stream` side, i.e.:
/// 1. Initiates graceful shutdown of `stream`
/// 2. Once done, initiates a graceful shutdown of `session`
/// 3. The function terminates, returning the number of bytes transferred in both directions.
#[cfg(feature = "runtime-tokio")]
pub async fn transfer_session<S>(
    session: &mut crate::Session,
    stream: &mut S,
    max_buffer: usize,
    abort_stream: Option<futures::future::AbortRegistration>,
) -> std::io::Result<(usize, usize)>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // We can use equally sized buffer for both directions
    tracing::debug!(
        session_id = ?session.id(),
        egress_buffer = max_buffer,
        ingress_buffer = max_buffer,
        "session buffers"
    );

    if let Some(abort_stream) = abort_stream {
        // We only allow aborting from the "stream" side, not from the "session side"
        // This is useful for UDP-like streams on the "stream" side, which cannot be terminated
        // by a signal from outside (e.g.: for TCP sockets such signal is socket closure).
        let (_, dummy) = futures::future::AbortHandle::new_pair();
        hopr_network_types::utils::copy_duplex_abortable(
            session,
            stream,
            (max_buffer, max_buffer),
            (dummy, abort_stream),
        )
        .await
        .map(|(a, b)| (a as usize, b as usize))
    } else {
        hopr_network_types::utils::copy_duplex(session, stream, (max_buffer, max_buffer))
            .await
            .map(|(a, b)| (a as usize, b as usize))
    }
}

/// This function will use the given generator to generate an initial seeding key.
/// It will check whether the given cache already contains a value for that key, and if not,
/// calls the generator (with the previous value) to generate a new seeding key and retry.
/// The function either finds a suitable free slot, inserting the `value` and returns the found key,
/// or terminates with `None` when `gen` returns the initial seed again.
pub(crate) async fn insert_into_next_slot<K, V, F>(
    cache: &moka::future::Cache<K, V>,
    generator: F,
    value: V,
) -> Option<K>
where
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(Option<K>) -> K,
{
    cache.run_pending_tasks().await;

    let initial = generator(None);
    let mut next = initial;
    loop {
        let insertion_result = cache
            .entry(next)
            .and_try_compute_with(|e| {
                if e.is_none() {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Put(value.clone()))
                } else {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Nop)
                }
            })
            .await;

        // If we inserted successfully, break the loop and return the insertion key
        if let Ok(moka::ops::compute::CompResult::Inserted(_)) = insertion_result {
            return Some(next);
        }

        // Otherwise, generate the next key
        next = generator(Some(next));

        // If generated keys made it to full loop, return failure
        if next == initial {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[tokio::test]
    async fn test_insert_into_next_slot() -> anyhow::Result<()> {
        let cache = moka::future::Cache::new(10);

        for i in 0..5 {
            let v = insert_into_next_slot(&cache, |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0), "foo".to_string())
                .await
                .ok_or(anyhow!("should insert"))?;
            assert_eq!(v, i);
            assert_eq!(Some("foo".to_string()), cache.get(&i).await);
        }

        assert!(
            insert_into_next_slot(&cache, |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0), "foo".to_string())
                .await
                .is_none(),
            "must not find slot when full"
        );

        Ok(())
    }
}
