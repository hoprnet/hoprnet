use std::time::Duration;

use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt};
use hopr_async_runtime::AbortHandle;
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_network_types::prelude::DestinationRouting;
use hopr_protocol_app::prelude::ApplicationData;
use tracing::{debug, error};

use crate::{
    SessionId,
    balancer::{RateController, RateLimitStreamExt, SurbControllerWithCorrection},
    errors::TransportSessionError,
    types::HoprStartProtocol,
};

/// Convenience function to copy data in both directions between a [`Session`](crate::Session) and arbitrary
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

pub(crate) fn spawn_keep_alive_stream<S>(
    session_id: SessionId,
    sender: S,
    routing: DestinationRouting,
) -> (SurbControllerWithCorrection, AbortHandle)
where
    S: futures::Sink<(DestinationRouting, ApplicationData)> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let elem = HoprStartProtocol::KeepAlive(session_id.into());

    // The stream is suspended until the caller sets a rate via the Controller
    let controller = RateController::new(0, Duration::from_secs(1));

    let (ka_stream, abort_handle) =
        futures::stream::abortable(futures::stream::repeat(elem).rate_limit_with_controller(&controller));

    let sender_clone = sender.clone();
    let fwd_routing_clone = routing.clone();

    // This task will automatically terminate once the returned abort handle is used.
    debug!(%session_id, "spawning keep-alive stream");
    hopr_async_runtime::prelude::spawn(
        ka_stream
            .map(move |msg| ApplicationData::try_from(msg).map(|m| (fwd_routing_clone.clone(), m)))
            .map_err(TransportSessionError::from)
            .try_for_each_concurrent(None, move |msg| {
                let mut sender_clone = sender_clone.clone();
                async move {
                    sender_clone
                        .send(msg)
                        .await
                        .map_err(|e| TransportSessionError::PacketSendingError(e.to_string()))
                }
            })
            .then(move |res| {
                match res {
                    Ok(_) => debug!(%session_id, "keep-alive stream done"),
                    Err(error) => error!(%session_id, %error, "keep-alive stream failed"),
                }
                futures::future::ready(())
            }),
    );

    // Currently, a keep-alive message can bear `HoprPacket::MAX_SURBS_IN_PACKET` SURBs,
    // so the correction by this factor is applied.
    (
        SurbControllerWithCorrection(controller, HoprPacket::MAX_SURBS_IN_PACKET as u32),
        abort_handle,
    )
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
