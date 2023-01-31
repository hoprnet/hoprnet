use core::panic;
use std::{collections::HashMap, pin::Pin, u8};

use futures::{
    channel::mpsc,
    ready,
    stream::Stream,
    task::{Context, Poll},
    Future,
};
use pin_project_lite::pin_project;
use std::sync::{Arc, Mutex};
use std::task::Waker;

// cfg_if! {
//     if #[cfg(target = "wasm32")] {
//         use wasm_bindgen_futures::spawn_local as executor
//     } else {
//         // search for a non-wasm executor
//     }

// }
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

fn spawn<F>(fun: F)
where
    F: Future<Output = ()> + 'static,
{
    if cfg!(target = "wasm32") {
        wasm_bindgen_futures::spawn_local(fun);
    } else {
        todo!("Implement me");
        // std::thread::spawn(fun);
    }
}

struct PingSharedState {
    /// set to true once completed
    completed: bool,
    /// called to wake the Future
    maybe_waker: Option<Waker>,
    /// timestamp set at creation
    started_at: usize,
    /// timestamp set once completed
    completed_at: Option<usize>,
}
// }

pub struct PingFuture {
    shared_state: Arc<Mutex<PingSharedState>>,
}

impl Future for PingFuture {
    type Output = usize;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(shared_state.completed_at.unwrap() - shared_state.started_at)
        } else {
            shared_state.maybe_waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl PingFuture {
    pub fn new() -> Self {
        let shared_state = Arc::new(Mutex::new(PingSharedState {
            completed: false,
            maybe_waker: None,
            started_at: get_time(),
            completed_at: None,
        }));

        Self { shared_state }
    }

    pub fn wake(&self) -> () {
        let mut shared_state = self.shared_state.lock().unwrap();

        shared_state.completed_at.replace(get_time());

        if let Some(waker) = shared_state.maybe_waker.take() {
            waker.wake()
        }
    }
}

struct StreamSharedState<St> {
    can_yield: bool,

    next_stream: Option<St>,
    maybe_waker: Option<Waker>,
}

pub struct StreamStream<St> {
    shared_state: Arc<Mutex<StreamSharedState<St>>>,
}

impl<St> StreamStream<St>
where
    St: Stream,
{
    fn new() -> StreamStream<St> {
        let shared_state = Arc::new(Mutex::new(StreamSharedState::<St> {
            can_yield: false,
            next_stream: None,
            maybe_waker: None,
        }));

        Self { shared_state }
    }

    fn take_stream(&self, new_stream: St) -> Result<(), String> {
        let mut shared_state = self.shared_state.lock().unwrap();

        // new_stream.filter(f)
        if shared_state.can_yield {
            Err(format!(
                "Cannot take stream because previous stream has not yet been consumed"
            ))
        } else {
            shared_state.can_yield = true;
            shared_state.next_stream.replace(new_stream);

            if let Some(waker) = shared_state.maybe_waker.take() {
                waker.wake();
            }

            Ok(())
        }
    }
}

impl<St> Stream for StreamStream<St>
where
    St: Stream,
{
    type Item = St;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();

        shared_state.maybe_waker.replace(cx.waker().clone());

        if shared_state.can_yield {
            shared_state.can_yield = false;
            Poll::Ready(shared_state.next_stream.take())
        } else {
            Poll::Pending
        }
    }
}

pin_project! {
    struct Server<St> {
        #[pin]
        stream: Option<St>,
        #[pin]
        next_stream: StreamStream<St>,
        status_messages_rx: mpsc::UnboundedReceiver<Box<[u8]>>,
        status_messages_tx: mpsc::UnboundedSender<Box<[u8]>>,
        ping_requests: std::collections::HashMap<u32, PingFuture>
    }
}

impl<St> Server<St>
where
    St: Stream + Unpin,
    St::Item: Into<Result<Box<[u8]>, String>>,
{
    fn new(stream: St) -> Self {
        let (status_messages_tx, status_messages_rx) = mpsc::unbounded::<Box<[u8]>>();
        Self {
            stream: Some(stream),
            next_stream: StreamStream::new(),
            status_messages_rx,
            status_messages_tx,
            ping_requests: HashMap::new(),
        }
    }

    async fn ping(&mut self) {
        let random_value: u32 = 0;

        self.ping_requests.insert(random_value, PingFuture::new());
    }

    fn update(self: Pin<&mut Self>, new_stream: St) -> Result<(), String> {
        let this = self.project();

        this.next_stream.take_stream(new_stream)
    }
}

impl<St> Stream for Server<St>
where
    St: Stream + Unpin,
    St::Item: Into<Result<Box<[u8]>, String>>,
{
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Poll::Ready(Some(new_stream)) = this.next_stream.as_mut().poll_next(cx) {
                this.stream.replace(new_stream);
                break Some(Ok(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ])));
            } else if let Some(item) =
                ready!(this.stream.as_mut().as_pin_mut().unwrap().poll_next(cx))
            {
                // TODO no error handling
                let msg = <St::Item as Into<Result<Box<[u8]>, String>>>::into(item).unwrap();

                match msg.first() {
                    Some(prefix) if *prefix == MessagePrefix::ConnectionStatus as u8 => {
                        match msg.get(1) {
                            Some(inner_prefix)
                                if [
                                    ConnectionStatusMessage::Stop as u8,
                                    ConnectionStatusMessage::Upgraded as u8,
                                ]
                                .contains(inner_prefix) =>
                            {
                                break None
                            }
                            Some(_) => break Some(Ok(msg)),
                            None => break None,
                        };
                    }
                    Some(prefix) if *prefix == MessagePrefix::StatusMessage as u8 => {
                        match msg.get(1) {
                            Some(inner_prefix) if *inner_prefix == StatusMessage::Ping as u8 => {
                                match this.status_messages_tx.poll_ready(cx) {
                                    Poll::Ready(Ok(())) => {
                                        match this.status_messages_tx.start_send(msg) {
                                            Ok(()) => (),
                                            Err(e) => panic!("{}", e),
                                        }
                                    }
                                    Poll::Ready(Err(e)) => panic!("{}", e),
                                    Poll::Pending => return Poll::Pending,
                                };
                            }
                            Some(inner_prefix) if *inner_prefix == StatusMessage::Pong as u8 => {
                                // 2 byte prefix, 4 byte ping identifier
                                if msg.len() < 6 {
                                    // drop malformed pong message
                                    return Poll::Pending;
                                }

                                let (_, ping_id) = msg.split_at(2);
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
                        break Some(Ok(msg))
                    }
                    // Empty message, stream ended
                    None => break None,
                    // TODO log this
                    _ => break None,
                };
                // this.handle_message(Some(item))
            } else {
                break None;
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.as_ref().unwrap().size_hint()
    }
}

pub mod wasm {
    use futures::{Stream, StreamExt};
    use js_sys::AsyncIterator;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;

    #[wasm_bindgen]
    pub struct RelayServer {
        server: Box<dyn Stream<Item = Result<Box<[u8]>, String>>>,
    }

    // pub impl RelayServer {
    //     fn new(stream: AsyncIterator) -> Self {

    //         Self {
    //             server: Box::new(super::Server::new(stream)),
    //         }
    //     }
    // }
}
