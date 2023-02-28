use std::{collections::HashMap, pin::Pin, u8};

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, FutureExt, Sink, SinkExt,
};
use pin_project_lite::pin_project;
use std::sync::{Arc, Mutex};
use std::task::Waker;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "wasm")]
use crate::streaming_iterable::{JsStreamingIterable, StreamingIterable};

use wasm_bindgen_futures::future_to_promise;

#[repr(u8)]
pub enum MessagePrefix {
    Payload = 0x00,
    StatusMessage = 0x01,
    WebRTC = 0x02,
    ConnectionStatus = 0x03,
}

#[repr(u8)]
pub enum StatusMessage {
    Ping = 0x00,
    Pong = 0x01,
}

#[repr(u8)]
pub enum ConnectionStatusMessage {
    Stop = 0x00,
    Restart = 0x01,
    Upgraded = 0x02,
}

fn get_time() -> usize {
    if cfg!(feature = "wasm32") {
        js_sys::Date::now() as u32 as usize
    } else {
        todo!("Implement me");
    }
}

pin_project! {
    pub struct PingFuture {
        waker: Option<Waker>,
        started_at: usize,
        completed_at: Option<usize>
    }
}

impl Future for PingFuture {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.completed_at {
            Some(completed_at) => Poll::Ready(*completed_at - *this.started_at),
            None => {
                // maybe turn this into a vec to store multiple waker instances
                *this.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl PingFuture {
    fn new() -> Self {
        Self {
            waker: None,
            started_at: get_time(),
            completed_at: None,
        }
    }

    /// Wake the future and assign final timestamp
    fn wake(&mut self) -> () {
        self.completed_at.get_or_insert(get_time());

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

pin_project! {
    pub struct NextStream {
        next_stream: Option<StreamingIterable>,
        waker: Option<Waker>,
    }
}

impl<'a> NextStream {
    fn new() -> NextStream {
        Self {
            next_stream: None,
            waker: None,
        }
    }

    fn take_stream(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
        match self.next_stream {
            Some(_) => Err(format!(
                "Cannot take stream because previous stream has not yet been consumed"
            )),
            None => {
                self.next_stream = Some(StreamingIterable::from(new_stream));

                if let Some(waker) = self.waker.take() {
                    waker.wake();
                }

                Ok(())
            }
        }
    }
}

impl<'a> Stream for NextStream {
    type Item = StreamingIterable;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.next_stream.take() {
            Some(stream) => Poll::Ready(Some(stream)),
            None => {
                // maybe turn this into a vec to store multiple waker instances
                *this.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pin_project! {
    struct Server {
        #[pin]
        stream: Option<StreamingIterable>,
        #[pin]
        next_stream: NextStream,
        #[pin]
        status_messages_rx: Option<UnboundedReceiver<Box<[u8]>>>,
        status_messages_tx: UnboundedSender<Box<[u8]>>,
        status_message_waker: Option<Waker>,
        ping_requests: std::collections::HashMap<u32, PingFuture>,
        buffered: Option<Box<[u8]>>
    }
}

impl Server {
    fn new(stream: StreamingIterable) -> Server {
        let (status_messages_tx, status_messages_rx) = mpsc::unbounded::<Box<[u8]>>();
        Self {
            stream: Some(stream),
            next_stream: NextStream::new(),
            status_messages_rx: Some(status_messages_rx),
            status_messages_tx,
            status_message_waker: None,
            ping_requests: HashMap::new(),
            buffered: None,
        }
    }

    /// Used to test whether the connection is alive
    async fn ping(&mut self) -> usize {
        let random_value: u32 = 0;

        let fut = PingFuture::new();
        // takes ownership
        self.ping_requests.insert(random_value, fut);

        // cannot clone futures
        self.ping_requests.get_mut(&random_value).unwrap().await
    }

    /// Used to attach a new incoming connection
    fn update(self: Pin<&mut Self>, new_stream: JsStreamingIterable) -> Result<(), String> {
        let mut this = self.project();

        this.next_stream.take_stream(new_stream)
    }
}

impl<'b> Stream for Server {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            // Make sure previous send attempt has finished
            // if let Some(x) = this.sink_send_future.as_mut().as_pin_mut() {
            //     ready!(x.poll(cx));
            // }

            if let Poll::Ready(Some(new_stream)) = this.next_stream.as_mut().poll_next(cx) {
                this.stream.set(Some(new_stream));
                break Some(Ok(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ])));
            } else if let Some(item) =
                ready!(this.stream.as_mut().as_pin_mut().unwrap().poll_next(cx))
            {
                let item = item.unwrap();
                match item.get(0) {
                    Some(prefix) if *prefix == MessagePrefix::ConnectionStatus as u8 => {
                        match item.get(1) {
                            Some(inner_prefix)
                                if [
                                    ConnectionStatusMessage::Stop as u8,
                                    ConnectionStatusMessage::Upgraded as u8,
                                ]
                                .contains(inner_prefix) =>
                            {
                                this.status_messages_tx.send(item).poll_unpin(cx);

                                break None;
                            }
                            Some(_) => break Some(Ok(item)),
                            None => break None,
                        };
                    }
                    Some(prefix) if *prefix == MessagePrefix::StatusMessage as u8 => {
                        match item.get(1) {
                            Some(inner_prefix) if *inner_prefix == StatusMessage::Ping as u8 => {
                                match this.status_messages_tx.poll_ready(cx) {
                                    Poll::Ready(Ok(())) => {
                                        this.status_messages_tx.send(item).poll_unpin(cx);
                                    }
                                    Poll::Ready(Err(e)) => panic!("{}", e),
                                    Poll::Pending => return Poll::Pending,
                                };
                            }
                            Some(inner_prefix) if *inner_prefix == StatusMessage::Pong as u8 => {
                                // 2 byte prefix, 4 byte ping identifier
                                if item.len() < 6 {
                                    // drop malformed pong message
                                    return Poll::Pending;
                                }

                                let (_, ping_id) = item.split_at(2);
                                match this
                                    .ping_requests
                                    .get_mut(&u32::from_ne_bytes(ping_id.try_into().unwrap()))
                                {
                                    Some(x) => {
                                        x.wake();
                                    }
                                    None => return Poll::Pending,
                                };
                            }
                            _ => panic!("invalid message"),
                        };
                    }
                    Some(prefix)
                        if [MessagePrefix::Payload as u8, MessagePrefix::WebRTC as u8]
                            .contains(prefix) =>
                    {
                        break Some(Ok(item))
                    }
                    // Empty message, stream ended
                    None => break None,
                    // TODO log this
                    _ => break None,
                };
            } else {
                break None;
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.as_ref().unwrap().size_hint()
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        self.stream.as_ref().unwrap().is_terminated()
    }
}

impl Sink<Box<[u8]>> for Server {
    type Error = String;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        if let Poll::Ready(Some(new_stream)) = this.next_stream.as_mut().poll_next(cx) {
            this.stream.set(Some(new_stream));
        }

        // Send all pending status messages before forwarding any new messages
        loop {
            match ready!(this
                .status_messages_rx
                .as_mut()
                .as_pin_mut()
                .unwrap()
                .poll_next(cx))
            {
                Some(item) => {
                    if item.starts_with(&[MessagePrefix::ConnectionStatus as u8])
                        && [
                            ConnectionStatusMessage::Stop as u8,
                            ConnectionStatusMessage::Restart as u8,
                            ConnectionStatusMessage::Upgraded as u8,
                        ]
                        .contains(item.get(1).unwrap())
                    {
                        this.stream.as_mut().as_pin_mut().unwrap().poll_close(cx);
                        return Poll::Ready(Err("closed".into()));
                    }
                    this.stream
                        .as_mut()
                        .as_pin_mut()
                        .unwrap()
                        .start_send(item)
                        .unwrap();

                    match this.stream.as_mut().as_pin_mut().unwrap().poll_flush(cx) {
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                        // FIXME
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    };
                }
                None => break,
            };
        }

        this.stream.as_mut().as_pin_mut().unwrap().poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
        let mut this = self.project();

        match item.get(1) {
            // Some(prefix) if *prefix == MessagePrefix::ConnectionStatus as u8 && item.len() > 1 && *(item.get(1).unwrap()) == ConnectionStatusMessage::Stop as u8 => {
            //     this.stream.as_mut().as_pin_mut().unwrap().poll_close(

            //     )
            // },
            Some(_) => this.stream.as_mut().as_pin_mut().unwrap().start_send(item),
            None => Ok(()),
        }
        // todo!()
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        this.stream.as_mut().as_pin_mut().unwrap().poll_close(cx)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        this.stream.as_mut().as_pin_mut().unwrap().poll_flush(cx)
    }
}

// #[cfg(feature = "wasm")]
// pub mod wasm {
//     use super::IntoSink;
//     use futures::{SinkExt, StreamExt};
//     use js_sys::AsyncIterator;
//     use utils_misc::{
//         async_iterable::wasm::{to_box_u8_stream, to_jsvalue_stream},
//         ok_or_jserr,
//     };
//     use wasm_bindgen::prelude::*;
//     use wasm_bindgen_futures::stream::JsStream;

//     #[wasm_bindgen]
//     pub struct Source {}

//     #[wasm_bindgen]
//     pub struct RelayServer {
//         server: super::Server<JsStream>,
//     }

//     #[wasm_bindgen]
//     impl RelayServer {
//         #[wasm_bindgen]
//         pub async fn next(&mut self) -> Result<JsValue, JsValue> {
//             to_jsvalue_stream(<super::Server<JsStream> as StreamExt>::next(&mut self.server).await)
//         }

//         #[wasm_bindgen]
//         pub async fn sink(&mut self, stream: AsyncIterator) -> Result<(), JsValue> {
//             let mut stream = JsStream::from(stream).map(to_box_u8_stream);

//             ok_or_jserr!(
//                 <super::Server<JsStream> as SinkExt<Box<[u8]>>>::send_all(
//                     &mut self.server,
//                     &mut stream
//                 )
//                 .await
//             )
//         }
//     }

//     #[wasm_bindgen]
//     pub fn relay_context(
//         stream: AsyncIterator,
//         sink: &js_sys::Function,
//     ) -> Result<RelayServer, JsValue> {
//         let stream = JsStream::from(stream);

//         let server = super::Server::new(stream, sink);

//         Ok(RelayServer { server })
//     }
// }
