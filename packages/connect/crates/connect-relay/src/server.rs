use core::pin::Pin;
use std::collections::HashMap;

use crate::{constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT, traits::DuplexStream};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::{select, Either},
    pin_mut,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, FutureExt, Sink, SinkExt, TryFutureExt,
};
use getrandom::getrandom;
use pin_project_lite::pin_project;
use std::task::Waker;
use utils_log::{error, info};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[repr(u8)]
pub enum MessagePrefix {
    Payload = 0x00,
    StatusMessage = 0x01,
    ConnectionStatus = 0x02,
    WebRTC = 0x03,
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

pub trait RelayServerCbs {
    fn on_close(&self) -> () {}
    fn on_upgrade(&self) -> () {}
}

pin_project! {
    /// In order to test the liveness of a relayed connection, the relay management
    /// code issues ping requests, which are encaspsulated in this struct.
    ///
    /// The struct itself implements the `futures::Future` trait, so it can get
    /// polled to see if there has been a reply.
    #[derive(Debug)]
    pub struct PingFuture {
        // used to signal that the ping request was successful
        waker: Option<Waker>,
        // timestamp when the request has been started, used check
        // timeout and compute latency
        started_at: u64,
        // filled with a timestamp once reply came back.
        completed_at: Option<u64>
    }
}

impl PingFuture {
    /// Creates a new ping request instance
    fn new() -> Self {
        Self {
            waker: None,
            started_at: current_timestamp(),
            completed_at: None,
        }
    }

    /// Once the ping reply came back, wake the future to tell
    /// the calling code that ping request has been successful.
    fn wake(&mut self) -> () {
        self.completed_at.get_or_insert(current_timestamp());

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl Future for PingFuture {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.completed_at {
            Some(completed_at) => Poll::Ready(*completed_at - *this.started_at),
            None => {
                *this.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pin_project! {
    /// Relayed connections consist of two individual connections,
    /// one to the initiator and another one to the destination.
    /// Both sides may reconnect multiple times, which creates a
    /// stream of *fresh* connections.
    ///
    /// This struct holds new incoming stream, e.g. after reconnects
    /// and itself implements the `futures::Stream` trait
    #[derive(Debug)]
    pub struct NextStream<St> {
        next_stream: Option<St>,
        waker: Option<Waker>,
    }
}

impl<'a, St> NextStream<St> {
    fn new() -> NextStream<St> {
        Self {
            next_stream: None,
            waker: None,
        }
    }

    /// Once there has been a reconnect, there is a *fresh* connection. This
    /// method takes this *fresh* connection and signals that a new stream
    /// item is ready to be processed.
    ///
    /// The method throws if the previous stream has not yet been taken out.
    fn take_stream(&mut self, new_stream: St) -> Result<(), String> {
        match self.next_stream {
            Some(_) => {
                error!("Cannot take stream because previous stream has not yet been consumed");
                Err(format!(
                    "Cannot take stream because previous stream has not yet been consumed"
                ))
            }
            None => {
                self.next_stream = Some(new_stream);

                info!("next_stream waker {:?}", self.waker);
                if let Some(waker) = self.waker.take() {
                    info!("waking next_stream stream");
                    waker.wake();
                }

                Ok(())
            }
        }
    }
}

impl<'a, St> Stream for NextStream<St> {
    type Item = St;

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
    ///  Encapsulates the relay-side stream state management.
    ///
    /// ┌────┐   stream    ┌────────┐
    /// │ A  ├────────────►│        │ stream
    /// └────┘             │Server  ├───────►
    ///                ┌──►│        │
    /// ┌────┐         │   └────────┘
    /// │ A' ├─────────┘
    /// └────┘  new stream
    ///
    /// At the beginning, this code get instantiated with a connection
    /// to one of the participants. Once there is a reconnects, a new
    /// connection can get attached and the server does a stream handover
    /// to the newly attached connection.
    #[derive(Debug)]
    struct Server<St: DuplexStream, Cbs: RelayServerCbs> {
        // the underlying Stream / Sink struct
        // FIXME: make it work for pure Rust *and* WebAssembly
        #[pin]
        stream: Option<St>,
        #[pin]
        // stream of new Stream / Sink structs, used
        // for stream handovers after reconnects
        next_stream: NextStream<St>,
        // used to dequeue status messages
        #[pin]
        status_messages_rx: Option<UnboundedReceiver<Box<[u8]>>>,
        // used to queue status messages
        status_messages_tx: UnboundedSender<Box<[u8]>>,
        // holds current ping requests
        ping_requests: std::collections::HashMap<[u8; 4], PingFuture>,
        // true if stream / sink has ended
        ended: bool,
        // set to true after stream switched
        stream_switched: bool,
        // holds a status message if it could not get send directly
        buffered_status_message: Option<Box<[u8]>>,
        // unique id of this instance
        id: Box<[u8; 4]>,
        // wake stream after reconnect
        poll_next_waker: Option<Waker>,
        // wake sink after reconnect
        poll_ready_waker: Option<Waker>,
        // signal events to management code
        callbacks: Option<Cbs>
    }
}

impl<St: DuplexStream, Cbs: RelayServerCbs> Server<St, Cbs> {
    /// Takes a stream to one of the endpoints and callbacks to the relay
    /// management code and creates a new instance server-side relay stream
    /// state management instance.
    fn new(stream: St, callbacks: Cbs) -> Server<St, Cbs> {
        let (status_messages_tx, status_messages_rx) = mpsc::unbounded::<Box<[u8]>>();

        let mut id = [0u8; 4];
        match getrandom(&mut id) {
            Ok(_) => (),
            Err(e) => panic!("Could not generate random identifier {}", e),
        };

        Self {
            stream: Some(stream),
            next_stream: NextStream::new(),
            status_messages_rx: Some(status_messages_rx),
            status_messages_tx,
            ping_requests: HashMap::new(),
            ended: false,
            stream_switched: false,
            buffered_status_message: None,
            id: Box::from(id),
            poll_next_waker: None,
            poll_ready_waker: None,
            callbacks: Some(callbacks),
        }
    }

    /// Used to test whether the relayed connection is alive, takes
    /// an optional custom timeout. If none is supplied, a default
    /// timeout is used.
    async fn ping(&mut self, maybe_timeout: Option<u64>) -> Result<u64, String> {
        let random_value: [u8; 4] = [0u8; 4];

        // FIXME: enable once client code is migrated
        // getrandom(&mut random_value).unwrap();

        let fut = PingFuture::new();
        // takes ownership
        self.ping_requests.insert(random_value, fut);

        let timeout_duration = maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64);

        let send_fut = self
            .send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| format!("Failed sending ping request {}", e));

        let send_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();

        pin_mut!(send_timeout);

        match select(send_timeout, send_fut).await {
            Either::Left(_) => return Err(format!("Low-level ping timed out after {} ms", timeout_duration)),
            Either::Right((Ok(()), _)) => (),
            Either::Right((Err(e), _)) => return Err(e),
        };

        // TODO: move this up to catch all
        let response_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();
        // cannot clone futures
        let ping = self.ping_requests.get_mut(&random_value).unwrap();

        pin_mut!(response_timeout);

        match select(response_timeout, ping).await {
            Either::Left(_) => Err(format!("Low-level ping timed out after {}", timeout_duration)),
            Either::Right((latency, _)) => Ok(latency),
        }
    }

    /// To test if the relayed connection is alive, the relay management code
    /// issues low-level ping requests. Each ping request gets a unique identifier.
    /// This method returns all currently open ping requests, mainly used for testing.
    pub fn get_pending_ping_requests(&self) -> Vec<[u8; 4]> {
        self.ping_requests.keys().copied().collect()
    }

    /// A relayed connection contains two sides, one towards the initiator and one
    /// towards the destination. Each side can break. Once one side reconnects, this
    /// method takes the *fresh* connections and attaches it to the existing relay
    /// connection pipeline.
    pub fn update(&mut self, new_stream: St) -> Result<(), String> {
        let res = self.next_stream.take_stream(new_stream);
        if let Some(item) = self.poll_next_waker.take() {
            item.wake()
        }

        if let Some(item) = self.poll_ready_waker.take() {
            item.wake()
        }

        res
    }

    /// Prepends logs with a per-instance identifier to enhance debugging
    pub fn log(&self, msg: &str) {
        info!("{} {}", hex::encode(*self.id), msg)
    }

    /// Prepends error logs with a per-instance identifier to enhance debugging
    pub fn error(&self, msg: &str) {
        error!("{} {}", hex::encode(*self.id), msg)
    }
}

impl<'b, St: DuplexStream, Cbs: RelayServerCbs> Stream for Server<St, Cbs> {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        *this.poll_next_waker = Some(cx.waker().clone());
        Poll::Ready(loop {
            if *this.ended {
                break None;
            }
            if let Poll::Ready(Some(new_stream)) = this.next_stream.as_mut().poll_next(cx) {
                // There has been a reonnect attempt, so assign a new stream
                this.stream.set(Some(new_stream));
                break Some(Ok(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ])));
            }

            if *this.stream_switched {
                // Stream switched but Sink was triggered first
                *this.stream_switched = false;
                break Some(Ok(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ])));
            }

            if this.stream.is_none() {
                return Poll::Pending;
            }

            match this.stream.as_mut().as_pin_mut().unwrap().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(item)) => {
                    let item = match item {
                        Ok(good) => good,
                        Err(some_err) => {
                            error!("stream failed due to {:?}", some_err);
                            // Stream threw an unrecoverable error, wait for better connection
                            this.stream.take();
                            return Poll::Ready(None);
                        }
                    };
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
                                    break if *inner_prefix == ConnectionStatusMessage::Stop as u8 {
                                        // Connection has ended, mark it ended for next iteration
                                        this.callbacks.as_mut().unwrap().on_close();
                                        *this.ended = true;
                                        Some(Ok(item))
                                    } else {
                                        this.callbacks.as_mut().unwrap().on_upgrade();
                                        // swallow message and go for next one
                                        continue;
                                    };
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
                                            match this.status_messages_tx.unbounded_send(item) {
                                                Ok(()) => (),
                                                Err(e) => {
                                                    error!("Failed queuing status message {}", e);
                                                    panic!()
                                                }
                                            };
                                        }
                                        Poll::Ready(Err(e)) => panic!("{}", e),
                                        Poll::Pending => return Poll::Pending,
                                    };
                                }
                                Some(inner_prefix) if *inner_prefix == StatusMessage::Pong as u8 => {
                                    // 2 byte prefix, 4 byte ping identifier

                                    let has_legacy_entry = this.ping_requests.contains_key(&[0u8; 4]);
                                    if item.len() < 6 && !has_legacy_entry {
                                        // drop malformed pong message
                                        return Poll::Pending;
                                    }

                                    let ping_id: [u8; 4] = if has_legacy_entry {
                                        info!("received legacy PONG {:?}", item);
                                        [0u8; 4]
                                    } else {
                                        info!("received PONG {:?}", item);
                                        item[2..6].try_into().unwrap()
                                    };

                                    match this.ping_requests.get_mut(&ping_id) {
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
                            if *prefix == MessagePrefix::Payload as u8 || *prefix == MessagePrefix::WebRTC as u8 =>
                        {
                            break Some(Ok(item))
                        }
                        // Empty message, stream ended
                        None => break None,
                        // TODO log this
                        _ => break None,
                    };
                }
                Poll::Ready(None) => {
                    break None;
                }
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.as_ref().unwrap().size_hint()
    }
}

impl<St: DuplexStream + FusedStream, Cbs: RelayServerCbs> FusedStream for Server<St, Cbs> {
    fn is_terminated(&self) -> bool {
        self.stream.as_ref().unwrap().is_terminated()
    }
}

impl<St: DuplexStream, Cbs: RelayServerCbs> Sink<Box<[u8]>> for Server<St, Cbs> {
    type Error = String;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>> {
        let mut this = self.project();

        *this.poll_ready_waker = Some(cx.waker().clone());

        match this.next_stream.as_mut().poll_next(cx) {
            Poll::Ready(new_stream) => {
                *this.stream_switched = true;
                this.stream.set(new_stream)
            }
            Poll::Pending => {
                if this.stream.is_none() {
                    return Poll::Pending;
                }
            }
        };

        if this.stream.is_none() {
            return Poll::Pending;
        }

        // Send all pending status messages before forwarding any new messages
        loop {
            let status_message_to_send = if let Some(buffered) = this.buffered_status_message.take() {
                buffered
            } else {
                match this.status_messages_rx.as_mut().as_pin_mut().unwrap().poll_next(cx) {
                    Poll::Pending => break,
                    Poll::Ready(None) => {
                        error!("status message stream is closed");
                        break;
                    }
                    Poll::Ready(Some(item)) => item,
                }
            };

            if status_message_to_send.len() == 0 {
                error!("fatal error: prevented sending of empty status message");
                break;
            }

            match status_message_to_send[0] {
                prefix if prefix == MessagePrefix::ConnectionStatus as u8 => {
                    if status_message_to_send.len() < 1 {
                        error!("fatal error: missing bytes in status message");
                        break;
                    }
                    if [
                        ConnectionStatusMessage::Stop as u8,
                        ConnectionStatusMessage::Restart as u8,
                        ConnectionStatusMessage::Upgraded as u8,
                    ]
                    .contains(status_message_to_send.get(1).unwrap())
                    {
                        this.stream.as_mut().as_pin_mut().unwrap().close();
                        return Poll::Ready(Err("closed".into()));
                    }
                }
                _ => {
                    match this.stream.as_mut().as_pin_mut().unwrap().poll_ready(cx) {
                        Poll::Pending => {
                            *this.buffered_status_message = Some(status_message_to_send);
                            return Poll::Pending;
                        }
                        Poll::Ready(_) => (),
                    };

                    match this
                        .stream
                        .as_mut()
                        .as_pin_mut()
                        .unwrap()
                        .start_send(status_message_to_send)
                    {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Could not send status message, {}", e);
                            return Poll::Ready(Err(e));
                        }
                    };

                    match this.stream.as_mut().as_pin_mut().unwrap().poll_flush(cx) {
                        Poll::Pending => return Poll::Pending,
                        // FIXME
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    };
                }
            }
        }

        match this.stream.as_mut().as_pin_mut().unwrap().poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => {
                error!("error while starting sink stream {}, deleting stream", e);
                this.stream.take();
                Poll::Pending
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String> {
        let mut this = self.project();

        if item.len() > 0 {
            this.stream.as_mut().as_pin_mut().unwrap().start_send(item)
        } else {
            error!("server: prevented sending empty message");
            Err("Message must not be empty".into())
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>> {
        let mut this = self.project();

        // The stream might have already ended, so stream is already deleted
        if let Some(stream) = this.stream.as_mut().as_pin_mut() {
            return stream.poll_close(cx);
        }

        Poll::Ready(Ok(()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>> {
        let mut this = self.project();

        // The stream might have already ended, so stream is already deleted
        if let Some(stream) = this.stream.as_mut().as_pin_mut() {
            return stream.poll_flush(cx);
        }

        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::streaming_iterable::{AsyncIterable, JsStreamingIterable, StreamingIterable, Uint8ArrayIteratorNext};
    use futures::{stream::Next, FutureExt, SinkExt, StreamExt};
    use js_sys::{AsyncIterator, Function, Number, Object, Promise, Reflect, Symbol, Uint8Array};
    use utils_misc::async_iterable::wasm::to_jsvalue_stream;

    use utils_log::info;
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type RelayServerOpts;

        #[wasm_bindgen(getter, js_name = "relayFreeTimeout")]
        pub fn relay_free_timeout() -> u32;
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
    extern "C" {
        #[wasm_bindgen]
        #[derive(Clone, Debug)]
        pub type RelayServerCbs;

        #[wasm_bindgen(js_name = "onClose", structural, method)]
        pub fn js_on_close(this: &RelayServerCbs);

        #[wasm_bindgen(js_name = "onUpgrade", structural, method)]
        pub fn js_on_upgrade(this: &RelayServerCbs);
    }

    impl super::RelayServerCbs for RelayServerCbs {
        fn on_close(&self) -> () {
            self.js_on_close()
        }

        fn on_upgrade(&self) -> () {
            self.js_on_upgrade()
        }
    }

    /// Wraps a server instance to make its API accessible by Javascript.
    ///
    /// This especially means turning the `futures::Stream` trait implemntation
    /// ```rust
    /// use std::pin::Pin;
    /// use std::task::{Context, Poll};
    /// trait Stream {
    ///     type Item;
    ///     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Box<[u8]>>>;
    /// }
    /// ```
    /// futures::Sink` trait implementation
    /// ```rust
    /// use std::pin::Pin;
    /// use std::task::{Context, Poll};
    /// pub trait Sink<Item> {
    ///   fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>>;
    ///   fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String>;
    ///   fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>>;
    ///   fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>>;
    /// }
    /// ```
    ///  into the following Typescript API
    ///
    /// ```ts
    /// interface IStream {
    ///   sink(source: AsyncIterable<Uint8Array>): Promise<void>;
    ///   source: AsyncIterable<Uint8Array>;
    /// ]
    /// ```
    #[wasm_bindgen]
    pub struct Server {
        w: super::Server<StreamingIterable, RelayServerCbs>,
    }

    #[wasm_bindgen]
    impl Server {
        /// Assign duplex stream endpoint and create a new instance
        /// of server-side relay stream state management code
        #[wasm_bindgen(constructor)]
        pub fn new(stream: JsStreamingIterable, signals: RelayServerCbs, _options: RelayServerOpts) -> Self {
            Self {
                w: super::Server::new(StreamingIterable::from(stream), signals),
            }
        }

        /// After a reconnect, assign a new stream
        #[wasm_bindgen]
        pub fn update(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
            self.w.update(StreamingIterable::from(new_stream))
        }

        /// Issues a new low-level ping request
        #[wasm_bindgen]
        pub fn ping(&mut self, timeout: Option<Number>) -> Promise {
            let this = unsafe { std::mem::transmute::<&mut Server, &mut Server>(self) };

            wasm_bindgen_futures::future_to_promise(this.w.ping(timeout.map(|f| f.value_of() as u64)).map(
                |r| match r {
                    Ok(u) => Ok(JsValue::from(u)),
                    Err(e) => Err(JsValue::from(e)),
                },
            ))
        }

        /// Expose identifiers of open ping request, mainly used for unit testing
        #[wasm_bindgen(getter, js_name = "pendingPingRequests")]
        pub fn get_pending_ping_requests(&self) -> Box<[Uint8Array]> {
            Box::from_iter(
                self.w
                    .get_pending_ping_requests()
                    .iter()
                    .map(|u| Uint8Array::from(&u[..])),
            )
        }

        /// Turns `futures::Stream` trait implementation into
        /// ```ts
        /// type Stream {
        ///   source: AsyncIterable<Uint8Array>;
        /// }
        /// ```
        #[wasm_bindgen(getter)]
        pub fn source(&mut self) -> AsyncIterable {
            let this = unsafe { std::mem::transmute::<&mut Self, &mut Self>(self) };
            let iterator_obj = Object::new();

            let iterator_fn = Closure::<dyn FnMut() -> Promise>::new(move || {
                let promise = Promise::new(&mut |resolve, reject| {
                    let fut = unsafe {
                        std::mem::transmute::<
                            Next<'_, super::Server<StreamingIterable, RelayServerCbs>>,
                            Next<'_, super::Server<StreamingIterable, RelayServerCbs>>,
                        >(this.w.next())
                    };
                    let fut = fut.map(to_jsvalue_stream).then(|x| async move {
                        resolve.call1(&JsValue::undefined(), &x.unwrap());
                    });

                    wasm_bindgen_futures::spawn_local(fut);
                });

                promise
            });

            // {
            //    next(): Promise<IteratorResult> {
            //      // ... function body
            //    }
            // }
            Reflect::set(&iterator_obj, &"next".into(), iterator_fn.as_ref()).unwrap();

            // Release closure to JS garbage collector
            iterator_fn.forget();

            let iterable_fn = Closure::once(move || iterator_obj);

            let iterable_obj = Object::new();

            // {
            //    [Symbol.aysncIterator](): Iterator {
            //      // ... function body
            //    }
            // }
            Reflect::set(
                &iterable_obj,
                &Symbol::async_iterator(),
                // Cast Closure to js_sys::Function
                &iterable_fn.as_ref().unchecked_ref(),
            )
            .unwrap();

            // Release closure to JS garbage collector
            iterable_fn.forget();

            // We just created the right object, no need to check this
            iterable_obj.unchecked_into()
        }

        /// Turns `futures::Sink<Box<[u8]>>` trait implementation into
        /// ```ts
        /// interface IStream {
        ///   sink(source: AsyncIterable<Uint8Array>): Promise<void>;
        /// }
        /// ```
        #[wasm_bindgen]
        pub fn sink(&mut self, source: AsyncIterable) -> js_sys::Promise {
            let promise = Promise::new(&mut |resolve, reject| {
                let this = unsafe { std::mem::transmute::<&mut Self, &mut Self>(self) };

                let async_sym = Symbol::async_iterator();

                let async_iter_fn = match Reflect::get(&source, async_sym.as_ref()) {
                    Ok(x) => x,
                    Err(e) => {
                        self.w
                            .error(format!("Error access Symbol.asyncIterator {:?}", e).as_str());
                        reject.call1(&JsValue::undefined(), &JsValue::from(e)).unwrap();
                        return;
                    }
                };

                let async_iter_fn: Function = match async_iter_fn.dyn_into() {
                    Ok(fun) => fun,
                    Err(e) => {
                        self.w
                            .error(format!("Cannot perform dynamic convertion {:?}", e).as_str());
                        reject.call1(&JsValue::undefined(), &JsValue::from(e)).unwrap();
                        return;
                    }
                };

                let async_it: AsyncIterator = match async_iter_fn.call0(&source).unwrap().dyn_into() {
                    Ok(x) => x,
                    Err(e) => {
                        self.w.error(format!("Cannot call iterable function {:?}", e).as_str());
                        reject.call1(&JsValue::undefined(), &JsValue::from(e)).unwrap();
                        return;
                    }
                };

                wasm_bindgen_futures::spawn_local(async move {
                    loop {
                        match async_it.next().map(JsFuture::from) {
                            Ok(chunk_fut) => {
                                // Initiates call to underlying JS functions
                                let chunk = match chunk_fut.await {
                                    Ok(x) => x,
                                    Err(e) => {
                                        this.w.error(format!("error handling next() future {:?}", e).as_str());
                                        this.w.stream.as_mut().unwrap().close().await;
                                        reject.call1(&JsValue::undefined(), &JsValue::from(e)).unwrap();
                                        return;
                                    }
                                };
                                let next = chunk.unchecked_into::<Uint8ArrayIteratorNext>();
                                if next.done() {
                                    resolve.call0(&JsValue::undefined());
                                    this.w.close().await;
                                    break;
                                } else {
                                    this.w.send(next.value()).await;
                                }
                            }
                            Err(e) => {
                                resolve.call0(&JsValue::undefined());

                                this.w.log(format!("Error calling next function {:?}", e).as_str());
                                this.w.close().await;
                            }
                        };
                    }
                });
            });

            promise
        }
    }
}
