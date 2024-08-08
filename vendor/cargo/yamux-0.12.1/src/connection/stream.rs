// Copyright (c) 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

use crate::frame::header::ACK;
use crate::{
    chunks::Chunks,
    connection::{self, StreamCommand},
    frame::{
        header::{Data, Header, StreamId, WindowUpdate},
        Frame,
    },
    Config, WindowUpdateMode, DEFAULT_CREDIT,
};
use futures::{
    channel::mpsc,
    future::Either,
    io::{AsyncRead, AsyncWrite},
    ready, SinkExt,
};
use parking_lot::{Mutex, MutexGuard};
use std::convert::TryInto;
use std::{
    fmt, io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

/// The state of a Yamux stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// Open bidirectionally.
    Open {
        /// Whether the stream is acknowledged.
        ///
        /// For outbound streams, this tracks whether the remote has acknowledged our stream.
        /// For inbound streams, this tracks whether we have acknowledged the stream to the remote.
        ///
        /// This starts out with `false` and is set to `true` when we receive or send an `ACK` flag for this stream.
        /// We may also directly transition:
        /// - from `Open` to `RecvClosed` if the remote immediately sends `FIN`.
        /// - from `Open` to `Closed` if the remote immediately sends `RST`.
        acknowledged: bool,
    },
    /// Open for incoming messages.
    SendClosed,
    /// Open for outgoing messages.
    RecvClosed,
    /// Closed (terminal state).
    Closed,
}

impl State {
    /// Can we receive messages over this stream?
    pub fn can_read(self) -> bool {
        !matches!(self, State::RecvClosed | State::Closed)
    }

    /// Can we send messages over this stream?
    pub fn can_write(self) -> bool {
        !matches!(self, State::SendClosed | State::Closed)
    }
}

/// Indicate if a flag still needs to be set on an outbound header.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Flag {
    /// No flag needs to be set.
    None,
    /// The stream was opened lazily, so set the initial SYN flag.
    Syn,
    /// The stream still needs acknowledgement, so set the ACK flag.
    Ack,
}

/// A multiplexed Yamux stream.
///
/// Streams are created either outbound via [`crate::Connection::poll_new_outbound`]
/// or inbound via [`crate::Connection::poll_next_inbound`].
///
/// `Stream` implements [`AsyncRead`] and [`AsyncWrite`] and also
/// [`futures::stream::Stream`].
pub struct Stream {
    id: StreamId,
    conn: connection::Id,
    config: Arc<Config>,
    sender: mpsc::Sender<StreamCommand>,
    flag: Flag,
    shared: Arc<Mutex<Shared>>,
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stream")
            .field("id", &self.id.val())
            .field("connection", &self.conn)
            .finish()
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Stream {}/{})", self.conn, self.id.val())
    }
}

impl Stream {
    pub(crate) fn new_inbound(
        id: StreamId,
        conn: connection::Id,
        config: Arc<Config>,
        credit: u32,
        sender: mpsc::Sender<StreamCommand>,
    ) -> Self {
        Self {
            id,
            conn,
            config: config.clone(),
            sender,
            flag: Flag::None,
            shared: Arc::new(Mutex::new(Shared::new(DEFAULT_CREDIT, credit, config))),
        }
    }

    pub(crate) fn new_outbound(
        id: StreamId,
        conn: connection::Id,
        config: Arc<Config>,
        window: u32,
        sender: mpsc::Sender<StreamCommand>,
    ) -> Self {
        Self {
            id,
            conn,
            config: config.clone(),
            sender,
            flag: Flag::None,
            shared: Arc::new(Mutex::new(Shared::new(window, DEFAULT_CREDIT, config))),
        }
    }

    /// Get this stream's identifier.
    pub fn id(&self) -> StreamId {
        self.id
    }

    pub fn is_write_closed(&self) -> bool {
        matches!(self.shared().state(), State::SendClosed)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.shared().state(), State::Closed)
    }

    /// Whether we are still waiting for the remote to acknowledge this stream.
    pub fn is_pending_ack(&self) -> bool {
        self.shared().is_pending_ack()
    }

    /// Set the flag that should be set on the next outbound frame header.
    pub(crate) fn set_flag(&mut self, flag: Flag) {
        self.flag = flag
    }

    pub(crate) fn shared(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock()
    }

    pub(crate) fn clone_shared(&self) -> Arc<Mutex<Shared>> {
        self.shared.clone()
    }

    fn write_zero_err(&self) -> io::Error {
        let msg = format!("{}/{}: connection is closed", self.conn, self.id);
        io::Error::new(io::ErrorKind::WriteZero, msg)
    }

    /// Set ACK or SYN flag if necessary.
    fn add_flag(&mut self, header: &mut Header<Either<Data, WindowUpdate>>) {
        match self.flag {
            Flag::None => (),
            Flag::Syn => {
                header.syn();
                self.flag = Flag::None
            }
            Flag::Ack => {
                header.ack();
                self.flag = Flag::None
            }
        }
    }

    /// Send new credit to the sending side via a window update message if
    /// permitted.
    fn send_window_update(&mut self, cx: &mut Context) -> Poll<io::Result<()>> {
        // When using [`WindowUpdateMode::OnReceive`] window update messages are
        // send early on data receival (see [`crate::Connection::on_frame`]).
        #[allow(deprecated)]
        if matches!(self.config.window_update_mode, WindowUpdateMode::OnReceive) {
            return Poll::Ready(Ok(()));
        }

        let mut shared = self.shared.lock();

        if let Some(credit) = shared.next_window_update() {
            ready!(self
                .sender
                .poll_ready(cx)
                .map_err(|_| self.write_zero_err())?);

            shared.window += credit;
            drop(shared);

            let mut frame = Frame::window_update(self.id, credit).right();
            self.add_flag(frame.header_mut());
            let cmd = StreamCommand::SendFrame(frame);
            self.sender
                .start_send(cmd)
                .map_err(|_| self.write_zero_err())?;
        }

        Poll::Ready(Ok(()))
    }
}

/// Byte data produced by the [`futures::stream::Stream`] impl of [`Stream`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Packet(Vec<u8>);

impl AsRef<[u8]> for Packet {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl futures::stream::Stream for Stream {
    type Item = io::Result<Packet>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if !self.config.read_after_close && self.sender.is_closed() {
            return Poll::Ready(None);
        }

        match self.send_window_update(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
            // Continue reading buffered data even though sending a window update blocked.
            Poll::Pending => {}
        }

        let mut shared = self.shared();

        if let Some(bytes) = shared.buffer.pop() {
            let off = bytes.offset();
            let mut vec = bytes.into_vec();
            if off != 0 {
                // This should generally not happen when the stream is used only as
                // a `futures::stream::Stream` since the whole point of this impl is
                // to consume chunks atomically. It may perhaps happen when mixing
                // this impl and the `AsyncRead` one.
                log::debug!(
                    "{}/{}: chunk has been partially consumed",
                    self.conn,
                    self.id
                );
                vec = vec.split_off(off)
            }
            return Poll::Ready(Some(Ok(Packet(vec))));
        }

        // Buffer is empty, let's check if we can expect to read more data.
        if !shared.state().can_read() {
            log::debug!("{}/{}: eof", self.conn, self.id);
            return Poll::Ready(None); // stream has been reset
        }

        // Since we have no more data at this point, we want to be woken up
        // by the connection when more becomes available for us.
        shared.reader = Some(cx.waker().clone());

        Poll::Pending
    }
}

// Like the `futures::stream::Stream` impl above, but copies bytes into the
// provided mutable slice.
impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if !self.config.read_after_close && self.sender.is_closed() {
            return Poll::Ready(Ok(0));
        }

        match self.send_window_update(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            // Continue reading buffered data even though sending a window update blocked.
            Poll::Pending => {}
        }

        // Copy data from stream buffer.
        let mut shared = self.shared();
        let mut n = 0;
        while let Some(chunk) = shared.buffer.front_mut() {
            if chunk.is_empty() {
                shared.buffer.pop();
                continue;
            }
            let k = std::cmp::min(chunk.len(), buf.len() - n);
            buf[n..n + k].copy_from_slice(&chunk.as_ref()[..k]);
            n += k;
            chunk.advance(k);
            if n == buf.len() {
                break;
            }
        }

        if n > 0 {
            log::trace!("{}/{}: read {} bytes", self.conn, self.id, n);
            return Poll::Ready(Ok(n));
        }

        // Buffer is empty, let's check if we can expect to read more data.
        if !shared.state().can_read() {
            log::debug!("{}/{}: eof", self.conn, self.id);
            return Poll::Ready(Ok(0)); // stream has been reset
        }

        // Since we have no more data at this point, we want to be woken up
        // by the connection when more becomes available for us.
        shared.reader = Some(cx.waker().clone());

        Poll::Pending
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        ready!(self
            .sender
            .poll_ready(cx)
            .map_err(|_| self.write_zero_err())?);
        let body = {
            let mut shared = self.shared();
            if !shared.state().can_write() {
                log::debug!("{}/{}: can no longer write", self.conn, self.id);
                return Poll::Ready(Err(self.write_zero_err()));
            }
            if shared.credit == 0 {
                log::trace!("{}/{}: no more credit left", self.conn, self.id);
                shared.writer = Some(cx.waker().clone());
                return Poll::Pending;
            }
            let k = std::cmp::min(shared.credit as usize, buf.len());
            let k = std::cmp::min(k, self.config.split_send_size);
            shared.credit = shared.credit.saturating_sub(k as u32);
            Vec::from(&buf[..k])
        };
        let n = body.len();
        let mut frame = Frame::data(self.id, body).expect("body <= u32::MAX").left();
        self.add_flag(frame.header_mut());
        log::trace!("{}/{}: write {} bytes", self.conn, self.id, n);

        // technically, the frame hasn't been sent yet on the wire but from the perspective of this data structure, we've queued the frame for sending
        // We are tracking this information:
        // a) to be consistent with outbound streams
        // b) to correctly test our behaviour around timing of when ACKs are sent. See `ack_timing.rs` test.
        if frame.header().flags().contains(ACK) {
            self.shared()
                .update_state(self.conn, self.id, State::Open { acknowledged: true });
        }

        let cmd = StreamCommand::SendFrame(frame);
        self.sender
            .start_send(cmd)
            .map_err(|_| self.write_zero_err())?;
        Poll::Ready(Ok(n))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.sender
            .poll_flush_unpin(cx)
            .map_err(|_| self.write_zero_err())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        if self.is_closed() {
            return Poll::Ready(Ok(()));
        }
        ready!(self
            .sender
            .poll_ready(cx)
            .map_err(|_| self.write_zero_err())?);
        let ack = if self.flag == Flag::Ack {
            self.flag = Flag::None;
            true
        } else {
            false
        };
        log::trace!("{}/{}: close", self.conn, self.id);
        let cmd = StreamCommand::CloseStream { ack };
        self.sender
            .start_send(cmd)
            .map_err(|_| self.write_zero_err())?;
        self.shared()
            .update_state(self.conn, self.id, State::SendClosed);
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub(crate) struct Shared {
    state: State,
    pub(crate) window: u32,
    pub(crate) credit: u32,
    pub(crate) buffer: Chunks,
    pub(crate) reader: Option<Waker>,
    pub(crate) writer: Option<Waker>,
    config: Arc<Config>,
}

impl Shared {
    fn new(window: u32, credit: u32, config: Arc<Config>) -> Self {
        Shared {
            state: State::Open {
                acknowledged: false,
            },
            window,
            credit,
            buffer: Chunks::new(),
            reader: None,
            writer: None,
            config,
        }
    }

    pub(crate) fn state(&self) -> State {
        self.state
    }

    /// Update the stream state and return the state before it was updated.
    pub(crate) fn update_state(
        &mut self,
        cid: connection::Id,
        sid: StreamId,
        next: State,
    ) -> State {
        use self::State::*;

        let current = self.state;

        match (current, next) {
            (Closed, _) => {}
            (Open { .. }, _) => self.state = next,
            (RecvClosed, Closed) => self.state = Closed,
            (RecvClosed, Open { .. }) => {}
            (RecvClosed, RecvClosed) => {}
            (RecvClosed, SendClosed) => self.state = Closed,
            (SendClosed, Closed) => self.state = Closed,
            (SendClosed, Open { .. }) => {}
            (SendClosed, RecvClosed) => self.state = Closed,
            (SendClosed, SendClosed) => {}
        }

        log::trace!(
            "{}/{}: update state: (from {:?} to {:?} -> {:?})",
            cid,
            sid,
            current,
            next,
            self.state
        );

        current // Return the previous stream state for informational purposes.
    }

    /// Calculate the number of additional window bytes the receiving side
    /// should grant the sending side via a window update message.
    ///
    /// Returns `None` if too small to justify a window update message.
    ///
    /// Note: Once a caller successfully sent a window update message, the
    /// locally tracked window size needs to be updated manually by the caller.
    pub(crate) fn next_window_update(&mut self) -> Option<u32> {
        if !self.state.can_read() {
            return None;
        }

        let new_credit = match self.config.window_update_mode {
            #[allow(deprecated)]
            WindowUpdateMode::OnReceive => {
                debug_assert!(self.config.receive_window >= self.window);

                self.config.receive_window.saturating_sub(self.window)
            }
            WindowUpdateMode::OnRead => {
                debug_assert!(self.config.receive_window >= self.window);
                let bytes_received = self.config.receive_window.saturating_sub(self.window);
                let buffer_len: u32 = self.buffer.len().try_into().unwrap_or(std::u32::MAX);

                bytes_received.saturating_sub(buffer_len)
            }
        };

        // Send WindowUpdate message when half or more of the configured receive
        // window can be granted as additional credit to the sender.
        //
        // See https://github.com/paritytech/yamux/issues/100 for a detailed
        // discussion.
        if new_credit >= self.config.receive_window / 2 {
            Some(new_credit)
        } else {
            None
        }
    }

    /// Whether we are still waiting for the remote to acknowledge this stream.
    pub fn is_pending_ack(&self) -> bool {
        matches!(
            self.state(),
            State::Open {
                acknowledged: false
            }
        )
    }
}
