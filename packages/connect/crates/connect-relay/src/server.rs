use std::{collections::HashMap, pin::Pin};

use crate::constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT;
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

#[cfg(feature = "wasm")]
use crate::streaming_iterable::{JsStreamingIterable, StreamingIterable};

#[cfg(feature = "wasm")]
use gloo_timers::future::sleep;
#[cfg(not(feature = "wasm"))]
use utils_misc::time::native::sleep;

#[cfg(not(feature = "wasm"))]
use utils_misc::time::native::current_timestamp;
#[cfg(feature = "wasm")]
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

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
extern "C" {
    #[wasm_bindgen]
    #[derive(Clone, Debug)]
    pub type RelayServerCbs;

    #[wasm_bindgen(js_name = "onClose", structural, method)]
    pub fn on_close(this: &RelayServerCbs);

    #[wasm_bindgen(js_name = "onUpgrade", structural, method)]
    pub fn on_upgrade(this: &RelayServerCbs);
}

pin_project! {
    /// Encapsulates everything needed to run a ping request
    #[derive(Debug)]
    pub struct PingFuture {
        waker: Option<Waker>,
        started_at: u64,
        completed_at: Option<u64>
    }
}

impl Future for PingFuture {
    type Output = u64;

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
            started_at: current_timestamp(),
            completed_at: None,
        }
    }

    /// Wake the future and assign final timestamp
    fn wake(&mut self) -> () {
        self.completed_at.get_or_insert(current_timestamp());

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

pin_project! {
    /// Used to call `poll_ready` on `Unpin` Sinks
    #[derive(Debug)]
    pub struct PollReady<'a, Si: ?Sized> {
        #[pin]
        sink: &'a mut Si,
    }

}

impl<'a, Si: Sink<Box<[u8]>> + Unpin + ?Sized> PollReady<'a, Si> {
    pub(super) fn new(sink: &'a mut Si) -> Self {
        Self { sink }
    }
}

impl<Si: Sink<Box<[u8]>> + Unpin + ?Sized> Future for PollReady<'_, Si> {
    type Output = Result<(), Si::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.sink.poll_ready(cx)
    }
}

pin_project! {
    /// Stream of Streams, returns a new stream
    /// on every reconnect
    #[derive(Debug)]
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

    /// Takes ownership of a new incoming duplex stream
    fn take_stream(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
        match self.next_stream {
            Some(_) => {
                error!("Cannot take stream because previous stream has not yet been consumed");
                Err(format!(
                    "Cannot take stream because previous stream has not yet been consumed"
                ))
            }
            None => {
                self.next_stream = Some(StreamingIterable::from(new_stream));

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

impl<'a> Stream for NextStream {
    type Item = StreamingIterable;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        info!("next stream, poll_next called");

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
    /// Struct that manages server-side relay connections
    /// and handles reconnects
    #[derive(Debug)]
    struct Server {
        // the underlying Stream / Sink struct
        #[pin]
        stream: Option<StreamingIterable>,
        #[pin]
        // stream of new Stream / Sink structs, used
        // for stream handovers after reconnects
        next_stream: NextStream,
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
        callbacks: Option<RelayServerCbs>
    }
}

impl Server {
    fn new(stream: StreamingIterable, callbacks: RelayServerCbs) -> Server {
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

    /// Used to test whether the relayed connection is alive
    async fn ping(&mut self, maybe_timeout: Option<u64>) -> Result<u64, String> {
        self.log("server: ping called");

        let mut random_value = [0u8; 4];
        getrandom(&mut random_value).unwrap();

        let fut = PingFuture::new();
        // takes ownership
        self.ping_requests.insert(random_value, fut);

        let timeout_duration = if let Some(timeout) = maybe_timeout {
            timeout
        } else {
            DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64
        };

        let send_fut = self
            .send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| format!("Failed sending ping request {}", e));

        let send_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();

        pin_mut!(send_timeout);

        match select(send_timeout, send_fut).await {
            Either::Left(_) => {
                return Err(format!(
                    "Low-level ping timed out after {}",
                    timeout_duration
                ))
            }
            Either::Right((Ok(()), _)) => (),
            Either::Right((Err(e), _)) => return Err(e),
        };
        self.log("server: after sending PING message");

        // TODO: move this up to catch all
        let response_timeout =
            sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();
        // cannot clone futures
        let ping = self.ping_requests.get_mut(&random_value).unwrap();

        pin_mut!(response_timeout);

        match select(response_timeout, ping).await {
            Either::Left(_) => Err(format!(
                "Low-level ping timed out after {}",
                timeout_duration
            )),
            Either::Right((latency, _)) => Ok(latency),
        }
    }

    /// Returns identifiers of ping requests
    pub fn get_pending_ping_requests(&self) -> Vec<[u8; 4]> {
        self.ping_requests.keys().copied().collect()
    }

    /// Used to attach a new incoming connection
    pub fn update(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
        self.log(format!("server: Initiating stream handover").as_str());

        let res = self.next_stream.take_stream(new_stream);
        if let Some(item) = self.poll_next_waker.take() {
            item.wake()
        }

        if let Some(item) = self.poll_ready_waker.take() {
            item.wake()
        }

        res
    }

    /// Prepends logs with a per-instance identifier
    pub fn log(&self, msg: &str) {
        info!("{} {}", hex::encode(*self.id), msg)
    }

    /// Prepends error logs with a per-instance identifier
    pub fn error(&self, msg: &str) {
        error!("{} {}", hex::encode(*self.id), msg)
    }
}

impl<'b> Stream for Server {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.log("server: poll_next called");

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
                info!("source: no stream set");
                return Poll::Pending;
            }

            match this.stream.as_mut().as_pin_mut().unwrap().poll_next(cx) {
                Poll::Pending => {
                    info!("server: stream pending");
                    return Poll::Pending;
                }
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
                    info!("item received {:?}", item);
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
                                Some(inner_prefix)
                                    if *inner_prefix == StatusMessage::Ping as u8 =>
                                {
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
                                Some(inner_prefix)
                                    if *inner_prefix == StatusMessage::Pong as u8 =>
                                {
                                    // 2 byte prefix, 4 byte ping identifier

                                    let has_legacy_entry =
                                        this.ping_requests.contains_key(&[0u8; 4]);
                                    if item.len() < 6 && !has_legacy_entry {
                                        // drop malformed pong message
                                        return Poll::Pending;
                                    }

                                    let ping_id = if has_legacy_entry {
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
                            if *prefix == MessagePrefix::Payload as u8
                                || *prefix == MessagePrefix::WebRTC as u8 =>
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
                    info!("stream ended");
                    break None;
                }
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

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), String>> {
        self.log("server: poll_ready called");
        let mut this = self.project();

        *this.poll_ready_waker = Some(cx.waker().clone());

        match this.next_stream.as_mut().poll_next(cx) {
            Poll::Ready(new_stream) => {
                info!("sink switched");
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
            let status_message_to_send = if let Some(buffered) = this.buffered_status_message.take()
            {
                buffered
            } else {
                match this
                    .status_messages_rx
                    .as_mut()
                    .as_pin_mut()
                    .unwrap()
                    .poll_next(cx)
                {
                    Poll::Pending => break,
                    Poll::Ready(None) => {
                        info!("status message stream is closed");
                        break;
                    }
                    Poll::Ready(Some(item)) => item,
                }
            };

            info!("status_message_to_send {:?}", status_message_to_send);

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
                        info!("restart received closed");
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

                    info!("initiated sending of status message");

                    match this.stream.as_mut().as_pin_mut().unwrap().poll_flush(cx) {
                        Poll::Pending => return Poll::Pending,
                        // FIXME
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    };
                    info!("status message flushed")
                }
            }
            info!("loop iteration");
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
        self.log(format!("server: start_send {:?}", item).as_str());

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
    use crate::streaming_iterable::{AsyncIterable, JsStreamingIterable, StreamingIterable};
    use futures::{stream::Next, FutureExt, SinkExt, StreamExt};
    use js_sys::{
        AsyncIterator, Function, IteratorNext, Number, Object, Promise, Reflect, Symbol, Uint8Array,
    };
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

    #[wasm_bindgen]
    pub struct Server {
        w: super::Server,
    }

    #[wasm_bindgen]
    impl Server {
        #[wasm_bindgen(constructor)]
        pub fn new(
            stream: JsStreamingIterable,
            signals: super::RelayServerCbs,
            _options: RelayServerOpts,
        ) -> Self {
            Self {
                w: super::Server::new(StreamingIterable::from(stream), signals),
            }
        }
        #[wasm_bindgen]
        pub fn update(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
            self.w.update(new_stream)
        }

        #[wasm_bindgen]
        pub fn ping(&mut self, timeout: Option<Number>) -> Promise {
            let this = unsafe { std::mem::transmute::<&mut Server, &mut Server>(self) };

            wasm_bindgen_futures::future_to_promise(
                this.w
                    .ping(timeout.map(|f| f.value_of() as u64))
                    .map(|r| match r {
                        Ok(u) => Ok(JsValue::from(u)),
                        Err(e) => Err(JsValue::from(e)),
                    }),
            )
        }

        #[wasm_bindgen(getter, js_name = "pendingPingRequests")]
        pub fn get_pending_ping_requests(&self) -> Box<[Uint8Array]> {
            Box::from_iter(
                self.w
                    .get_pending_ping_requests()
                    .iter()
                    .map(|u| unsafe { Uint8Array::view(u) }),
            )
        }

        #[wasm_bindgen(getter)]
        pub fn source(&mut self) -> AsyncIterable {
            self.w.log("source called");

            let this = unsafe { std::mem::transmute::<&mut Self, &mut Self>(self) };
            let iterator_obj = Object::new();

            let iterator_fn = Closure::<dyn FnMut() -> Promise>::new(move || {
                info!("rs: iterator code called");
                let promise = Promise::new(&mut |resolve, reject| {
                    let fut = unsafe {
                        std::mem::transmute::<Next<'_, super::Server>, Next<'_, super::Server>>(
                            this.w.next(),
                        )
                    };
                    let fut = fut.map(to_jsvalue_stream).then(|x| async move {
                        info!("source future executed");
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

        /// Takes a JS async iterable stream and feeds it into a Rust Sink
        #[wasm_bindgen]
        pub fn sink(&mut self, source: AsyncIterable) -> js_sys::Promise {
            self.w.log("sink called");

            let promise = Promise::new(&mut |resolve, reject| {
                // let this = self;
                let this = unsafe { std::mem::transmute::<&mut Self, &mut Self>(self) };

                let async_sym = Symbol::async_iterator();

                let async_iter_fn = match Reflect::get(&source, async_sym.as_ref()) {
                    Ok(x) => x,
                    Err(e) => {
                        self.w
                            .error(format!("Error access Symbol.asyncIterator {:?}", e).as_str());
                        reject
                            .call1(&JsValue::undefined(), &JsValue::from(e))
                            .unwrap();
                        return;
                    }
                };

                let async_iter_fn: Function = match async_iter_fn.dyn_into() {
                    Ok(fun) => fun,
                    Err(e) => {
                        self.w
                            .error(format!("Cannot perform dynamic convertion {:?}", e).as_str());
                        reject
                            .call1(&JsValue::undefined(), &JsValue::from(e))
                            .unwrap();
                        return;
                    }
                };

                let async_it: AsyncIterator = match async_iter_fn.call0(&source).unwrap().dyn_into()
                {
                    Ok(x) => x,
                    Err(e) => {
                        self.w
                            .error(format!("Cannot call iterable function {:?}", e).as_str());
                        reject
                            .call1(&JsValue::undefined(), &JsValue::from(e))
                            .unwrap();
                        return;
                    }
                };

                wasm_bindgen_futures::spawn_local(async move {
                    loop {
                        info!("iteration");
                        match async_it.next().map(JsFuture::from) {
                            Ok(chunk_fut) => {
                                // Initiates call to underlying JS functions
                                let chunk = match chunk_fut.await {
                                    Ok(x) => x,
                                    Err(e) => {
                                        this.w.error(
                                            format!("error handling next() future {:?}", e)
                                                .as_str(),
                                        );
                                        this.w.stream.as_mut().unwrap().close().await;
                                        reject
                                            .call1(&JsValue::undefined(), &JsValue::from(e))
                                            .unwrap();
                                        return;
                                    }
                                };
                                let next = chunk.unchecked_into::<IteratorNext>();
                                info!("sink: next chunk {:?}", next);
                                if next.done() {
                                    resolve.call0(&JsValue::undefined());
                                    this.w.close().await;
                                    break;
                                } else {
                                    this.w
                                        .send(Box::from_iter(
                                            next.value().dyn_into::<Uint8Array>().unwrap().to_vec(),
                                        ))
                                        .await;

                                    this.w.log("after sending");
                                }
                            }
                            Err(e) => {
                                resolve.call0(&JsValue::undefined());

                                this.w
                                    .log(format!("Error calling next function {:?}", e).as_str());
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
