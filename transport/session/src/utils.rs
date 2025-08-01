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
