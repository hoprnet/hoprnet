use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::Future,
    ready, Stream,
};
use libp2p::PeerId;
use pin_project_lite::pin_project;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll, Waker},
};
use utils_log::error;
use utils_misc::traits::DuplexStream;

static RELAY_CONNECTION_IDENTIFIER_SEPERATOR: &str = " <-> ";

#[derive(Copy, Clone, Eq)]
pub struct RelayConnectionIdentifier {
    id_a: PeerId,
    id_b: PeerId,
}

pub struct RelayConnectionEndpoints<St> {
    pub(crate) new_stream_a: UnboundedSender<St>,
    pub(crate) ping_a_tx: UnboundedSender<Box<[u8]>>,
    pub(crate) ping_a_rx: UnboundedReceiver<Box<[u8]>>,
    pub(crate) new_stream_b: UnboundedSender<St>,
    pub(crate) ping_b_tx: UnboundedSender<Box<[u8]>>,
    pub(crate) ping_b_rx: UnboundedReceiver<Box<[u8]>>,
}

impl ToString for RelayConnectionIdentifier {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}",
            self.id_a.to_string(),
            RELAY_CONNECTION_IDENTIFIER_SEPERATOR,
            self.id_b.to_string()
        )
    }
}

impl FromStr for RelayConnectionIdentifier {
    type Err = String;
    ///
    /// ```rust
    /// use connect_relay::server_new::RelayConnectionIdentifier;
    /// use libp2p::PeerId;
    /// use std::str::FromStr;
    ///
    /// const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    /// const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    ///
    /// let id: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    ///
    /// assert!(RelayConnectionIdentifier::from_str(id.to_string().as_str()).unwrap().eq(&id));
    /// ```
    fn from_str(s: &str) -> Result<Self, String> {
        let ids = s.split(RELAY_CONNECTION_IDENTIFIER_SEPERATOR).collect::<Vec<_>>();

        // A valid identifier contains exactly 2 PeerIds
        if ids.len() != 2 {
            return Err("Not a valid relay connection identifier".into());
        }

        let a = PeerId::from_str(ids[0]).map_err(|e| e.to_string())?;
        let b = PeerId::from_str(ids[1]).map_err(|e| e.to_string())?;

        RelayConnectionIdentifier::try_from((a, b))
    }
}

impl Hash for RelayConnectionIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id_a.hash(state);
        self.id_b.hash(state);
    }
}

impl PartialEq for RelayConnectionIdentifier {
    ///
    /// ```rust
    /// use connect_relay::server_new::RelayConnectionIdentifier;
    /// use libp2p::PeerId;
    /// use std::str::FromStr;
    ///
    /// const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    /// const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    ///
    /// let id: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    /// let id_copy: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    ///
    /// assert!(id.eq(&id_copy));
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.id_a.eq(&other.id_a) && self.id_b.eq(&other.id_b)
    }
}

impl TryFrom<(PeerId, PeerId)> for RelayConnectionIdentifier {
    type Error = String;

    fn try_from(val: (PeerId, PeerId)) -> Result<Self, Self::Error> {
        match val.0.cmp(&val.1) {
            Ordering::Equal => Err("Keys must not be equal".into()),
            Ordering::Greater => Ok(RelayConnectionIdentifier {
                id_a: val.0,
                id_b: val.1,
            }),
            Ordering::Less => Ok(RelayConnectionIdentifier {
                id_a: val.1,
                id_b: val.0,
            }),
        }
    }
}

#[repr(u8)]
pub enum MessagePrefix {
    /// Message is a payload and need to be forwarded
    Payload = 0x00,
    /// Message is a status message and handles the node <-> relay relationship
    StatusMessage = 0x01,
    /// Message is a connection status message and handles the state of the
    /// link between initiator and destination
    ConnectionStatus = 0x02,
    /// Message contains WebRTC signalling information and need to be forwarded
    WebRTC = 0x03,
}

#[repr(u8)]
pub enum StatusMessage {
    /// Used to see if stream counterparty is active
    Ping = 0x00,
    /// Used to signal to relay that counterparty is active
    Pong = 0x01,
}

#[repr(u8)]
pub enum ConnectionStatusMessage {
    /// Notifies the relay and the other relay end to end the stream
    Stop = 0x00,
    /// Notifies the other relay end about a reconnect.
    /// This is used to redo the end-to-end connection handshake.
    Restart = 0x01,
    /// Notifies the relay that the connection has been upgraded and the
    /// relayed connection is no longer needed
    Upgraded = 0x02,
}

pin_project! {
    /// Holds one end of a relayed connections
    pub struct End<St> {
        // Holds the stream
        // to be set and overwritten at runtime
        #[pin]
        st: Option<St>,
        // chunk fetched from stream
        // to be sent soon
        buffered: Option<Box<[u8]>>,
        // PeerId of the destination
        id: PeerId,
        ping_tx: UnboundedSender<Box<[u8]>>,
        #[pin]
        // receive status messages from other end
        ping_rx: UnboundedReceiver<Box<[u8]>>,
        // receive a new stream (e.g. after a reconnect)
        #[pin]
        new_stream_rx: UnboundedReceiver<St>,
        // if true, stream is about to end, i.e. poll_close has been called
        ending: bool,
        // if true, stream is done, no more data is expected
        ended: bool,
        // stores the latest ping timeout
        ping_timeout: Option<Pin<Box<dyn Future<Output = ()>>>>,
        // if there is a running ping request, store its start timestamp
        ping_start: Option<u64>,
        // once the pong response came back, store the current timestamp
        ping_ended: Option<u64>,
        // once the pong response came back, wake the thread to process the result
        ping_waker: Option<Waker>
    }
}

impl<St: DuplexStream> End<St> {
    pub fn new(
        st: St,
        id: PeerId,
        ping_tx: UnboundedSender<Box<[u8]>>,
        ping_rx: UnboundedReceiver<Box<[u8]>>,
        new_stream_rx: UnboundedReceiver<St>,
    ) -> Self {
        Self {
            st: Some(st),
            buffered: None,
            id,
            ping_rx,
            ping_tx,
            new_stream_rx,
            ending: false,
            ended: false,
            ping_timeout: None,
            ping_ended: None,
            ping_start: None,
            ping_waker: None,
        }
    }

    /// Consumes the underlying stream and handles protocol messages and
    /// forwards all relevant messages to their destination, including payload
    /// messages.
    ///
    /// Returns `Poll::Ready(Ok(None))` once stream has ended.
    pub fn poll_stream_done(
        self: Pin<&mut Self>,
        other: &mut Pin<&mut End<St>>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<()>, String>> {
        let mut this = self.project();

        if *this.ended {
            return Poll::Ready(Ok(None));
        }

        // 1. try to deliver queued message
        // 2. check for reconnected stream
        // 3. check for status messages to be sent
        // 4. check for recent incoming messages
        // -> repeat
        loop {
            if *this.ending {
                let other_st = Pin::new(other.st.as_mut().unwrap());
                match other_st.poll_close(cx) {
                    Poll::Ready(_) => {
                        *this.ended = true;
                        return Poll::Ready(Ok(None));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            if let Some(chunk) = this.buffered.take() {
                let mut other_st = Pin::new(other.st.as_mut().unwrap());
                match other_st.as_mut().poll_ready(cx)? {
                    Poll::Ready(()) => {
                        if chunk.len() == 2
                            && chunk[0] == MessagePrefix::ConnectionStatus as u8
                            && chunk[1] == ConnectionStatusMessage::Stop as u8
                        {
                            *this.ending = true;
                        }
                        other_st.as_mut().start_send(chunk).map_err(|e| e.to_string())?;

                        // We just sent a STOP message, so close the stream
                        if *this.ending == true {
                            continue;
                        }
                    }
                    Poll::Pending => {
                        // We didn't succeed in sending chunk,
                        // so keep it for a next attempt
                        *this.buffered = Some(chunk);
                        return Poll::Pending;
                    }
                };
            }

            if let Poll::Ready(Some(new_stream)) = this.new_stream_rx.as_mut().poll_next(cx) {
                this.st.set(Some(new_stream));

                // Drop any previously cached message
                *this.buffered = Some(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ]));
                continue;
            }

            if let Poll::Ready(Some(status_message)) = this.ping_rx.as_mut().poll_next(cx) {
                *this.buffered = Some(status_message);
                continue;
            }

            match ready!(this.st.as_mut().as_pin_mut().unwrap().poll_next(cx)?) {
                Some(chunk) => {
                    match chunk[0] {
                        x if x == MessagePrefix::ConnectionStatus as u8 => {
                            if chunk.len() < 2 {
                                error!("unrecognizable connection status message. message is missing second byte");
                                continue;
                            }

                            match chunk[1] {
                                y if y == ConnectionStatusMessage::Stop as u8 => {
                                    // Correct?

                                    *this.buffered = Some(chunk);

                                    continue;
                                }
                                y if y == ConnectionStatusMessage::Restart as u8 => {
                                    *this.buffered = Some(chunk);
                                    continue;
                                }
                                y if y == ConnectionStatusMessage::Upgraded as u8 => {
                                    // Swallow UPGRADED message for backwards-compatibility
                                    // we might want to free the slot automatically in the future
                                    continue;
                                }
                                y => error!("unrecognizable connection status message [{}]", y),
                            }
                        }
                        x if x == MessagePrefix::StatusMessage as u8 => {
                            if chunk.len() < 2 {
                                error!("unrecognizable status message. message is missing second byte");
                                continue;
                            }

                            match chunk[1] {
                                y if y == StatusMessage::Pong as u8 => {
                                    this.ping_tx.unbounded_send(chunk).unwrap();
                                    continue;
                                }
                                y if y == StatusMessage::Ping as u8 => {
                                    *this.buffered = Some(Box::new([
                                        MessagePrefix::StatusMessage as u8,
                                        StatusMessage::Pong as u8,
                                    ]));
                                    continue;
                                }
                                y => error!("Unrecognizable status message [{}]", y),
                            }
                        }
                        x if (x == MessagePrefix::Payload as u8 || x == MessagePrefix::WebRTC as u8) => {
                            *this.buffered = Some(chunk);
                            continue;
                        }
                        x => {
                            error!("Received unrecognizable message [{}]", x)
                        }
                    }
                }
                None => {
                    *this.ending = true;
                    continue;
                }
            }
        }
    }
}

pin_project! {
    /// Holds two streams that form a relayed connections
    /// from peer_a <-> relay <-> peer_b
    pub struct Server<St> {
        // stream from a->b
        #[pin]
        a: End<St>,
        // stream from b->a
        #[pin]
        b: End<St>,
    }
}

impl<St: DuplexStream> Server<St> {
    /// Creates a new relay connection
    ///
    /// This method will not `poll` the streams.
    pub fn new(stream_a: St, peer_a: PeerId, stream_b: St, peer_b: PeerId) -> (Self, RelayConnectionEndpoints<St>) {
        let (ping_a_receive_tx, ping_a_receive_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (ping_a_send_tx, ping_a_send_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (ping_b_receive_tx, ping_b_receive_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (ping_b_send_tx, ping_b_send_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (new_stream_a_tx, new_stream_a_rx) = mpsc::unbounded::<St>();
        let (new_stream_b_tx, new_stream_b_rx) = mpsc::unbounded::<St>();

        (
            Self {
                a: End::new(
                    stream_a,
                    peer_a.clone(),
                    ping_a_send_tx,
                    ping_a_receive_rx,
                    new_stream_a_rx,
                ),

                b: End::new(
                    stream_b,
                    peer_b.clone(),
                    ping_b_send_tx,
                    ping_b_receive_rx,
                    new_stream_b_rx,
                ),
            },
            RelayConnectionEndpoints {
                ping_a_rx: ping_a_send_rx,
                ping_a_tx: ping_a_receive_tx,
                ping_b_rx: ping_b_send_rx,
                ping_b_tx: ping_b_receive_tx,
                new_stream_a: new_stream_a_tx,
                new_stream_b: new_stream_b_tx,
            },
        )
    }

    /// Gets the id of the relay connections
    pub fn get_id(&self) -> RelayConnectionIdentifier {
        assert!(!self.a.id.eq(&self.b.id), "Identifier must not be equal");

        (self.a.id.clone(), self.b.id.clone()).try_into().unwrap()
    }
}

impl<St: DuplexStream> Future for Server<St> {
    type Output = Result<(), String>;

    /// Future that resolves once the connection has ended.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(loop {
            let mut pending = false;

            if !self.a.ended {
                let mut this = self.as_mut().project();

                match this.a.poll_stream_done(&mut this.b, cx) {
                    Poll::Pending => {
                        pending = true;
                    }
                    Poll::Ready(Err(e)) => {
                        pending = false;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        pending = false;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }

            if !self.b.ended {
                let mut this = self.as_mut().project();

                match this.b.poll_stream_done(&mut this.a, cx) {
                    Poll::Pending => {
                        pending = true;
                    }
                    Poll::Ready(Err(e)) => {
                        pending = false;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        pending = false;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }

            if pending {
                return Poll::Pending;
            }

            if self.a.ended && self.b.ended {
                break Ok(());
            }
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use futures::{stream::FusedStream, Sink, Stream, StreamExt};

//     use super::*;
//     use std::str::FromStr;

//     const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
//     const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";

//     pin_project! {
//         struct TestingDuplexStream {
//             #[pin]
//             rx: UnboundedReceiver<Box<[u8]>>,
//             #[pin]
//             tx: UnboundedSender<Box<[u8]>>,
//         }
//     }

//     impl TestingDuplexStream {
//         fn new() -> (Self, UnboundedSender<Box<[u8]>>, UnboundedReceiver<Box<[u8]>>) {
//             let (send_tx, send_rx) = mpsc::unbounded::<Box<[u8]>>();
//             let (receive_tx, receive_rx) = mpsc::unbounded::<Box<[u8]>>();

//             (
//                 Self {
//                     rx: send_rx,
//                     tx: receive_tx,
//                 },
//                 send_tx,
//                 receive_rx,
//             )
//         }
//     }

//     impl FusedStream for TestingDuplexStream {
//         fn is_terminated(&self) -> bool {
//             self.rx.is_terminated()
//         }
//     }

//     impl Stream for TestingDuplexStream {
//         type Item = Result<Box<[u8]>, String>;
//         fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//             let this = self.project();

//             match this.rx.poll_next(cx) {
//                 Poll::Pending => Poll::Pending,
//                 Poll::Ready(Some(x)) => Poll::Ready(Some(Ok(x))),
//                 Poll::Ready(None) => Poll::Ready(None),
//             }
//         }

//         fn size_hint(&self) -> (usize, Option<usize>) {
//             self.rx.size_hint()
//         }
//     }

//     impl Sink<Box<[u8]>> for TestingDuplexStream {
//         type Error = String;
//         fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_ready(cx).map_err(|e| e.to_string())
//         }

//         fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
//             let this = self.project();

//             this.tx.start_send(item).map_err(|e| e.to_string())
//         }

//         fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_flush(cx).map_err(|e| e.to_string())
//         }

//         fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_close(cx).map_err(|e| e.to_string())
//         }
//     }

//     impl DuplexStream for TestingDuplexStream {}

//     pin_project! {
//         struct TestingPoll< St> {
//             #[pin]
//             end: End<St>,
//             #[pin]
//             other_end:End<St>,
//         }
//     }

//     impl<St: DuplexStream> TestingPoll<St> {
//         fn new(end: End<St>, other_end: End<St>) -> Self {
//             Self { end, other_end }
//         }
//     }

//     impl<St: DuplexStream> Future for TestingPoll<St> {
//         type Output = ();
//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             let mut this = self.project();

//             match this.end.poll_stream_done(&mut this.other_end, cx) {
//                 Poll::Pending => Poll::Pending,
//                 Poll::Ready(_) => Poll::Ready(()),
//             }
//         }
//     }

//     pin_project! {
//         struct TestingPollActive<St> {
//             #[pin]
//             end: End<St>,
//             #[pin]
//             other_end:End<St>,
//             st_send: UnboundedSender<Box<[u8]>>,
//             st_other_send: UnboundedSender<Box<[u8]>>,
//             first_attempt: bool
//         }
//     }

//     impl<St: DuplexStream> TestingPollActive<St> {
//         fn new(
//             end: End<St>,
//             other_end: End<St>,
//             st_send: UnboundedSender<Box<[u8]>>,
//             st_other_send: UnboundedSender<Box<[u8]>>,
//         ) -> Self {
//             Self {
//                 end,
//                 other_end,
//                 st_send,
//                 st_other_send,
//                 first_attempt: true,
//             }
//         }
//     }

//     impl<St: DuplexStream> Future for TestingPollActive<St> {
//         type Output = ();
//         fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             let mut pending = false;

//             {
//                 let mut this = self.as_mut().project();

//                 match this.end.as_mut().poll_active(cx, None) {
//                     Poll::Pending => {
//                         if *this.first_attempt {
//                             *this.first_attempt = false;
//                             this.st_send
//                                 .unbounded_send(Box::new([
//                                     MessagePrefix::StatusMessage as u8,
//                                     StatusMessage::Pong as u8,
//                                 ]))
//                                 .unwrap();
//                         }

//                         pending = true;
//                     }
//                     Poll::Ready(_) => {
//                         this.st_send
//                             .unbounded_send(Box::new([
//                                 MessagePrefix::ConnectionStatus as u8,
//                                 ConnectionStatusMessage::Stop as u8,
//                             ]))
//                             .unwrap();

//                         this.st_other_send
//                             .unbounded_send(Box::new([
//                                 MessagePrefix::ConnectionStatus as u8,
//                                 ConnectionStatusMessage::Stop as u8,
//                             ]))
//                             .unwrap();
//                     }
//                 }
//             }

//             {
//                 let mut this = self.as_mut().project();

//                 if let Poll::Pending = this.other_end.poll_stream_done(&mut this.end, cx) {
//                     pending = true;
//                 }
//             }

//             {
//                 let mut this = self.as_mut().project();

//                 if let Poll::Pending = this.end.poll_stream_done(&mut this.other_end, cx) {
//                     pending = true;
//                 }
//             }

//             if pending {
//                 Poll::Pending
//             } else {
//                 Poll::Ready(())
//             }
//         }
//     }

//     #[test]
//     fn test_identifier() {
//         assert!(
//             (RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()))
//                 .is_ok())
//         );

//         assert!(
//             (RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(ALICE).unwrap()))
//                 .is_err())
//         );

//         let ab =
//             RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()))
//                 .unwrap();

//         let ba =
//             RelayConnectionIdentifier::try_from((PeerId::from_str(BOB).unwrap(), PeerId::from_str(ALICE).unwrap()))
//                 .unwrap();

//         assert!(ab.eq(&ba));
//         assert!(ab.to_string().eq(&ba.to_string()));
//     }

//     #[async_std::test]
//     async fn test_connection_end() {
//         let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
//         let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

//         let (st, st_send, _) = TestingDuplexStream::new();
//         let (st_other, _, st_other_receive) = TestingDuplexStream::new();

//         let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
//         let conn_b = End::new(
//             st_other,
//             PeerId::from_str(ALICE).unwrap().clone(),
//             status_ab_tx,
//             status_ba_rx,
//         );

//         st_send
//             .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
//             .unwrap();

//         st_send
//             .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]))
//             .unwrap();

//         st_send
//             .unbounded_send(Box::new([
//                 MessagePrefix::ConnectionStatus as u8,
//                 ConnectionStatusMessage::Stop as u8,
//             ]))
//             .unwrap();

//         TestingPoll::new(conn_a, conn_b).await;

//         assert_eq!(
//             st_other_receive.collect::<Vec<Box<[u8]>>>().await,
//             vec![
//                 Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]) as Box<[u8]>,
//                 Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>,
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>
//             ]
//         );
//     }

//     #[async_std::test]
//     async fn test_stream_upgrade() {
//         let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
//         let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

//         let (st, st_send, _) = TestingDuplexStream::new();
//         let (st_other, _, st_other_receive) = TestingDuplexStream::new();

//         let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
//         let conn_b = End::new(
//             st_other,
//             PeerId::from_str(ALICE).unwrap().clone(),
//             status_ab_tx,
//             status_ba_rx,
//         );

//         // Should not forward this message
//         st_send
//             .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
//             .unwrap();

//         let (st_next, st_next_send, _) = TestingDuplexStream::new();

//         assert!(conn_a.update(st_next).is_ok());

//         st_next_send
//             .unbounded_send(Box::new([
//                 MessagePrefix::ConnectionStatus as u8,
//                 ConnectionStatusMessage::Stop as u8,
//             ]))
//             .unwrap();

//         TestingPoll::new(conn_a, conn_b).await;

//         assert_eq!(
//             st_other_receive.collect::<Vec<Box<[u8]>>>().await,
//             vec![
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Restart as u8
//                 ]) as Box<[u8]>,
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>
//             ]
//         );
//     }

//     #[async_std::test]
//     async fn test_is_active() {
//         let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
//         let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

//         let (st, st_send, st_receive) = TestingDuplexStream::new();
//         let (st_other, st_other_send, _) = TestingDuplexStream::new();

//         let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
//         let conn_b = End::new(
//             st_other,
//             PeerId::from_str(ALICE).unwrap().clone(),
//             status_ab_tx,
//             status_ba_rx,
//         );

//         TestingPollActive::new(conn_a, conn_b, st_send, st_other_send).await;

//         assert_eq!(
//             st_receive.collect::<Vec<Box<[u8]>>>().await,
//             vec![
//                 Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8]) as Box<[u8]>,
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>
//             ]
//         );
//     }

//     #[async_std::test]
//     async fn test_server() {
//         let (st_a, st_a_send, st_a_receive) = TestingDuplexStream::new();
//         let (st_b, st_b_send, st_b_receive) = TestingDuplexStream::new();

//         let server = Server::new(
//             st_a,
//             PeerId::from_str(ALICE).unwrap(),
//             st_b,
//             PeerId::from_str(BOB).unwrap(),
//         );

//         st_a_send
//             .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
//             .unwrap();

//         st_a_send
//             .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]))
//             .unwrap();

//         st_a_send
//             .unbounded_send(Box::new([
//                 MessagePrefix::ConnectionStatus as u8,
//                 ConnectionStatusMessage::Stop as u8,
//             ]))
//             .unwrap();

//         st_b_send
//             .unbounded_send(Box::new([MessagePrefix::Payload as u8, 1, 1, 1, 1]))
//             .unwrap();

//         st_b_send
//             .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 1, 1, 1, 1]))
//             .unwrap();

//         st_b_send
//             .unbounded_send(Box::new([
//                 MessagePrefix::ConnectionStatus as u8,
//                 ConnectionStatusMessage::Stop as u8,
//             ]))
//             .unwrap();

//         assert!(server.await.is_ok());

//         assert_eq!(
//             st_a_receive.collect::<Vec<Box<[u8]>>>().await,
//             vec![
//                 Box::new([MessagePrefix::Payload as u8, 1, 1, 1, 1]) as Box<[u8]>,
//                 Box::new([MessagePrefix::WebRTC as u8, 1, 1, 1, 1]) as Box<[u8]>,
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>
//             ]
//         );

//         assert_eq!(
//             st_b_receive.collect::<Vec<Box<[u8]>>>().await,
//             vec![
//                 Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]) as Box<[u8]>,
//                 Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>,
//                 Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>
//             ]
//         );
//     }
// }
