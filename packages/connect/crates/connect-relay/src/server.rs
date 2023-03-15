use std::{collections::HashMap, pin::Pin, u8};

use crate::constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT;
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::select,
    future::Either,
    pin_mut, ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, FutureExt, Sink, SinkExt,
};
use pin_project_lite::pin_project;
use std::task::Waker;

#[cfg(feature = "wasm")]
use crate::streaming_iterable::{JsStreamingIterable, StreamingIterable};

use wasm_bindgen::prelude::*;

#[cfg(not(feature = "wasm"))]
use async_std::task::sleep;
#[cfg(not(feature = "wasm"))]
use utils_misc::time::native::current_timestamp;

#[cfg(feature = "wasm")]
use gloo_timers::future::sleep;
#[cfg(feature = "wasm")]
use utils_misc::time::wasm::current_timestamp;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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
    if cfg!(feature = "wasm") {
        js_sys::Date::now() as u32 as usize
    } else {
        todo!("Implement me");
    }
}

pin_project! {
    #[derive(Debug)]
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

// #[derive(Debug)]
// #[must_use = "futures do nothing unless you `.await` or poll them"]
pin_project! {
    pub struct PollReady<'a, Si: ?Sized> {
        #[pin]
        sink: &'a mut Si,
    }

}

// Pinning is never projected to children
// impl<Si: Unpin + ?Sized> Unpin for PollReady<'_, Si> {}

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
    async fn ping(&mut self, maybe_timeout: Option<usize>) -> Result<usize, String> {
        log("server: ping called");
        let random_value: u32 = 0;

        let fut = PingFuture::new();
        // takes ownership
        self.ping_requests.insert(random_value, fut);

        let timeout_duration = if let Some(timeout) = maybe_timeout {
            timeout
        } else {
            DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as usize
        };

        let send_fut = self
            .send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map(|r| match r {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Failed sending ping request {}", e)),
            });

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
        log("server: after sending PING message");

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

    pub fn get_pending_ping_requests(&self) -> Vec<u32> {
        let reqs: Vec<u32> = self.ping_requests.keys().copied().collect();
        log("requests");
        reqs
    }

    /// Used to attach a new incoming connection
    pub fn update(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
        // le/t mut this = self.project();

        self.next_stream.take_stream(new_stream)
    }
}

impl<'b> Stream for Server {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        log("server: poll_next called");
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
                                match this.status_messages_tx.unbounded_send(item) {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log(format!(
                                            "Failed queuing connection status request {}",
                                            e
                                        )
                                        .as_str());
                                        panic!()
                                    }
                                };

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
                                        match this.status_messages_tx.unbounded_send(item) {
                                            Ok(()) => (),
                                            Err(e) => {
                                                log(format!("Failed queuing status message {}", e)
                                                    .as_str());
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
                                log(format!("received PONG {:?}", item).as_str());

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
        log("server: poll_ready called");
        let mut this = self.project();

        if let Poll::Ready(Some(new_stream)) = this.next_stream.as_mut().poll_next(cx) {
            this.stream.set(Some(new_stream));
        }

        // Send all pending status messages before forwarding any new messages
        loop {
            let received = match this
                .status_messages_rx
                .as_mut()
                .as_pin_mut()
                .unwrap()
                .poll_next(cx)
            {
                Poll::Pending => break,
                Poll::Ready(t) => t,
            };

            log(format!("received {:?}", received).as_str());

            match received {
                Some(item) => {
                    if item.starts_with(&[MessagePrefix::ConnectionStatus as u8])
                        && [
                            ConnectionStatusMessage::Stop as u8,
                            ConnectionStatusMessage::Restart as u8,
                            ConnectionStatusMessage::Upgraded as u8,
                        ]
                        .contains(item.get(1).unwrap())
                    {
                        this.stream.as_mut().as_pin_mut().unwrap().close();
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
            log("loop iteration");
        }

        this.stream.as_mut().as_pin_mut().unwrap().poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
        log("server: start_send called");
        log(format!("server: start_send {:?}", item).as_str());

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

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::streaming_iterable::{AsyncIterable, JsStreamingIterable, StreamingIterable};
    use futures::{stream::Next, FutureExt, SinkExt, StreamExt};
    use js_sys::{
        AsyncIterator, Function, IteratorNext, Number, Object, Promise, Reflect, Symbol, Uint8Array,
    };
    use utils_misc::async_iterable::wasm::to_jsvalue_stream;

    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        #[derive(Clone, Debug)]
        pub type RelayServerCbs;

        #[wasm_bindgen]
        pub fn onClose();

        #[wasm_bindgen]
        pub fn onUpgrade();
    }

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
            _signals: RelayServerCbs,
            _options: RelayServerOpts,
        ) -> Self {
            Self {
                w: super::Server::new(StreamingIterable::from(stream)),
            }
        }
        #[wasm_bindgen]
        pub fn update(&mut self, new_stream: JsStreamingIterable) -> Result<(), String> {
            self.w.update(new_stream)
        }

        #[wasm_bindgen]
        pub fn ping(&mut self, timeout: Option<Number>) -> Promise {
            let this = unsafe { std::mem::transmute::<&mut Server, &mut Server>(self) };
            let ping_fut = this
                .w
                .ping(timeout.map(|f| f.value_of() as u32 as usize))
                .map(|x| match x {
                    Ok(u) => Ok(JsValue::from(u)),   // converting from usize
                    Err(e) => Err(JsValue::from(e)), // converting from string
                });

            wasm_bindgen_futures::future_to_promise(ping_fut)
        }

        #[wasm_bindgen(getter, js_name = "pendingPingRequests")]
        pub fn get_pending_ping_requests(&self) -> Box<[Number]> {
            Box::from_iter(
                self.w
                    .get_pending_ping_requests()
                    .iter()
                    .map(|u| Number::from(*u)),
            )
        }

        #[wasm_bindgen(getter)]
        pub fn source(&mut self) -> AsyncIterable {
            let this = unsafe { std::mem::transmute::<&mut Self, &'static mut Self>(self) };
            let iterator_obj = Object::new();

            log("source called");
            let iterator_fn = Closure::<dyn FnMut() -> Promise>::new(move || {
                let fut = unsafe {
                    std::mem::transmute::<Next<'_, super::Server>, Next<'_, super::Server>>(
                        this.w.next(),
                    )
                };
                log("rs: iterator code called");
                wasm_bindgen_futures::future_to_promise(async move {
                    log("source fut executed");
                    to_jsvalue_stream(fut.await)
                })
            });

            // {
            //    next(): Promise<IteratorResult> {
            //      // ... function body
            //    }
            // }
            Reflect::set(&iterator_obj, &"next".into(), iterator_fn.as_ref()).unwrap();

            // This leaks memory, but intended
            iterator_fn.forget();

            // let wrapped = Wrapper { js_output: this };
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

            // This leaks memory, but intended
            iterable_fn.forget();

            iterable_obj.dyn_into().unwrap()
        }

        #[wasm_bindgen]
        pub fn sink(&mut self, source: AsyncIterable) -> js_sys::Promise {
            log("sink called");

            let async_sym = Symbol::async_iterator();

            let async_iter_fn = match Reflect::get(&source, async_sym.as_ref()) {
                Ok(x) => x,
                Err(e) => {
                    log(format!("Error access Symbol.asyncIterator {:?}", e).as_str());
                    todo!()
                }
            };

            let async_iter_fn: Function = match async_iter_fn.dyn_into() {
                Ok(fun) => fun,
                Err(e) => {
                    log(format!("Cannot perform dynamic convertion {:?}", e).as_str());
                    todo!()
                }
            };

            let async_it: AsyncIterator = match async_iter_fn.call0(&source).unwrap().dyn_into() {
                Ok(x) => x,
                Err(e) => {
                    log(format!("Cannot call iterable function {:?}", e).as_str());
                    todo!()
                }
            };

            log(format!("iterator {:?}", async_it).as_str());

            // let foo = self;

            let this = unsafe { std::mem::transmute::<&mut Self, &mut Self>(self) };

            wasm_bindgen_futures::future_to_promise(async move {
                loop {
                    log("iteration");
                    match async_it.next().map(JsFuture::from) {
                        Ok(m) => {
                            log(format!("next future {:?}", m).as_str());
                            let foo = match m.await {
                                Ok(x) => x,
                                Err(e) => {
                                    log(format!("error handling next() future {:?}", e).as_str());
                                    this.w.stream.as_mut().unwrap().close().await;
                                    todo!()
                                    // break;
                                }
                            };
                            let next = foo.unchecked_into::<IteratorNext>();
                            log(format!("sink: next chunk {:?}", next).as_str());
                            if next.done() {
                                this.w.close().await;
                                todo!()
                            } else {
                                log("sending");
                                log(format!(
                                    "sending result {:?}",
                                    this.w
                                        .send(Box::from_iter(
                                            Uint8Array::new(&next.value()).to_vec()
                                        ))
                                        .await
                                )
                                .as_str());
                                log("after sending");
                            }
                            // break;
                        }
                        Err(e) => {
                            log(format!("Error calling next function {:?}", e).as_str());
                            this.w.close().await;
                            // break;
                        }
                    };
                }
            })
            // js_sys::Promise::resolve(&JsValue::from(""))

            // log("sink end");
        }
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
