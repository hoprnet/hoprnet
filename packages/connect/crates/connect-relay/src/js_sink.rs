use core::panic;
use std::{pin::Pin, u8};

use futures::{
    channel::mpsc,
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, Sink, SinkExt, StreamExt,
};
use js_sys::{AsyncIterator, Function, Object};
use pin_project_lite::pin_project;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use utils_misc::{async_iterable::wasm::to_jsvalue_stream, ok_or_jserr};
use wasm_bindgen::{closure, convert::IntoWasmAbi, prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

#[cfg(feature = "wasm")]
use wasm_bindgen::{closure::Closure, JsValue};

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
const IStream: &'static str = r#"
interface IStream {
    sink(source: AsyncIterable<Uint8Array>): Promise<void>;
    source: AsyncIterable<Uint8Array>;
}
"#;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // // The `console.log` is quite polymorphic, so we can bind it with multiple
    // // signatures. Note that we need to use `js_name` to ensure we always call
    // // `log` in JS.
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_u32(a: u32);

    // // Multiple arguments too!
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_many(a: &str, b: &str);
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MyJsSource {}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MyJsSource {
    async fn next(&mut self) -> Result<JsValue, JsValue> {
        todo!()
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
extern "C" {
    #[wasm_bindgen(is_type_of = AsyncIterable::looks_like_async_iterable, typescript_type = "Iterator<any>")]
    pub type AsyncIterable;
}

impl AsyncIterable {
    fn looks_like_async_iterable(it: &JsValue) -> bool {
        if !it.is_object() {
            return false;
        }

        let async_sym = js_sys::Symbol::async_iterator();
        let async_iter_fn = match js_sys::Reflect::get(it, async_sym.as_ref()) {
            Ok(f) => f,
            Err(_) => return false,
        };

        log(format!("{:?}", async_iter_fn).as_str());
        async_iter_fn.is_function()
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
extern "C" {
    #[wasm_bindgen(typescript_type = IStream)]
    #[derive(Clone, Debug)]
    pub type MyJsStream;

    #[wasm_bindgen(structural, method)]
    pub async fn sink(this: &MyJsStream, stream: &JsValue);

    #[wasm_bindgen(structural, method, getter)]
    pub fn source(this: &MyJsStream) -> JsValue;
}

// TODO: make this thread-safe using an Arc
pin_project! {
    pub struct MyJsStreamStruct<'a> {
        iter: Option<AsyncIterator>,
        done: bool,
        next: Option<JsFuture>,
        fut: Option<MyJsStreamStructFuture>,
        sink_flushed_fut: Option<Pin<Box<dyn Future<Output = ()>>>>,
        #[pin]
        js_stream: &'a MyJsStream,
        sink_called: bool,
        next_chunk: Option<Box<[u8]>>,
        waker: Option<std::task::Waker>,
    }
}

struct MyJsStreamStructFuture {
    js_stream: &'static MyJsStreamStruct<'static>,
}

impl Future for MyJsStreamStructFuture {
    type Output = Box<[u8]>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // let mut this = unsafe { self.get_unchecked_mut() };

        // foo.
        // if this.js_stream.next_chunk.is_none() {
        //     this.js_stream.waker = Some(cx.waker().clone());
        //     Poll::Pending
        // } else {
        //     let to_send = self.js_stream.next_chunk.take().unwrap();
        //     Poll::Ready(to_send)
        // }

        Poll::Pending
    }
}

// Turn this into an arc
impl MyJsStreamStruct<'_> {
    fn from<'a>(stream: &'a MyJsStream) -> MyJsStreamStruct<'a> {
        MyJsStreamStruct {
            js_stream: stream,
            done: false,
            next: None,
            iter: None,
            sink_called: false,
            fut: None,
            next_chunk: None,
            waker: None,
            sink_flushed_fut: None,
        }
    }
}

// #[wasm_bindgen]
// impl MyJsStreamStruct<'_> {
//     #[wasm_bindgen]
//     pub async fn next_item(&mut self) -> Result<JsValue, JsValue> {
//         todo!()
//         // to_jsvalue_stream(Some(Ok(self.js_stream.fut.unwrap().await)))
//     }
// }

impl Stream for MyJsStreamStruct<'_> {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        if self.iter.is_none() {
            let async_sym = js_sys::Symbol::async_iterator();

            let initial = self.js_stream.source();
            let async_iter_fn = match js_sys::Reflect::get(&initial, async_sym.as_ref()) {
                Ok(x) => x,
                Err(_) => return Poll::Ready(None),
            };

            let async_iter_fn: js_sys::Function = match async_iter_fn.dyn_into() {
                Ok(fun) => fun,
                Err(e) => {
                    log(format!("{:?}", e).as_str());
                    self.done = true;
                    return Poll::Ready(None);
                }
            };

            let async_it: AsyncIterator = match async_iter_fn.call0(&initial).unwrap().dyn_into() {
                Ok(x) => x,
                Err(e) => {
                    log(format!("{:?}", e).as_str());
                    self.done = true;
                    return Poll::Ready(None);
                }
            };

            self.iter.replace(async_it);
        }

        let future = match self.next.as_mut() {
            Some(val) => val,
            None => match self.iter.as_ref().unwrap().next().map(JsFuture::from) {
                Ok(val) => {
                    self.next = Some(val);
                    self.next.as_mut().unwrap()
                }
                Err(e) => {
                    self.done = true;
                    return Poll::Ready(Some(Err(format!("{:?}", e))));
                }
            },
        };

        match Pin::new(future).poll(cx) {
            Poll::Ready(res) => match res {
                Ok(iter_next) => {
                    let next = iter_next.unchecked_into::<js_sys::IteratorNext>();
                    if next.done() {
                        self.done = true;
                        Poll::Ready(None)
                    } else {
                        self.next.take();
                        Poll::Ready(Some(Ok(Box::from_iter(
                            js_sys::Uint8Array::new(&next.value()).to_vec(),
                        ))))
                    }
                }
                Err(e) => {
                    self.done = true;
                    Poll::Ready(Some(Err(format!("{:?}", e))))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FusedStream for MyJsStreamStruct<'_> {
    fn is_terminated(&self) -> bool {
        self.done
    }
}

impl Sink<Box<[u8]>> for MyJsStreamStruct<'_> {
    type Error = String;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        log("poll_ready called");

        let this = self.project();

        if !*this.sink_called {
            log("calling sink");
            // let foo = self.my_fun();
            // JsValue::from(Closure::new(self.my_fun()));
            // self.js_stream.sink(&JsValue::from(AsyncIterableHelper {
            //     js_stream: self.get_unchecked_mut(),
            // }));
            let other_closure: Closure<dyn FnMut() -> js_sys::Object> = Closure::new(|| {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
                js_sys::Reflect::set(&obj, &"value".into(), &JsValue::FALSE).unwrap();
                obj
            });
            let inner_obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &inner_obj,
                &"next".into(),
                other_closure.as_ref().unchecked_ref(),
            )
            .unwrap();

            let obj = js_sys::Object::new();
            let bar: Closure<dyn FnMut() -> js_sys::Object> = Closure::once(move || {
                log("interval elapsed!");
                inner_obj
            });
            // let closure = Closure::new(foo.next_item());
            // let abi = foo.into_abi();
            js_sys::Reflect::set(
                &obj,
                &js_sys::Symbol::async_iterator(),
                // Cast Closure to js_sys::Function
                bar.as_ref().unchecked_ref(),
            )
            .unwrap();

            *this.sink_flushed_fut = Some(Box::pin(Pin::new(this.js_stream).as_mut().sink(&obj)));

            // *this.sink_flushed_fut = Some(Box::pin(this.js_stream.sink(&obj)));

            *this.sink_called = true;
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
        log("poll_ready called");

        Ok(())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        match this.sink_flushed_fut.unwrap().as_mut().poll(cx) {
            Poll::Ready(x) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }

        // let foo = Box::pin(this.sink_flushed_fut.as_mut().unwrap().as_mut())
        //         match this.sink_flushed_fut.unwrap().as_mut().poll(cx) {
        //             Poll::Ready(()) => Poll::Ready(Ok(())),
        //             Poll::Pending => Poll::Pending
        //         }
    }
}

#[wasm_bindgen]
pub async fn foo_bar(stream: MyJsStream) {
    let mut foo = MyJsStreamStruct::from(&stream);

    ok_or_jserr!(foo.send(Box::new([0u8, 0u8])).await);
    // log(format!("first result {:?}", foo.next().await).as_str());
    // log(format!("second result {:?}", foo.next().await).as_str());
    // log(format!("third result {:?}", foo.next().await).as_str());
}

// trait IntoSink<T, Sink> {
//     fn into_sink(t: T) -> Sink;
// }

// impl<T, Sink> IntoSink<T, Sink> for T
// where
//     T: Into<Sink>,
// {
//     fn into_sink(t: T) -> Sink {
//         t.into()
//     }
// }

// impl IntoSink<&js_sys::Function, JsSink> for &js_sys::Function {
//     fn into_sink(t: &js_sys::Function) -> JsSink {
//         JsSink::new(t)
//     }
// }

// struct JsSinkSharedState {
//     message_available: bool,
//     ended: bool,
//     sink_attached: bool,
//     message_waker: Option<Waker>,
//     ready_to_send_waker: Option<Waker>,
//     sink_attached_waker: Option<Waker>,
//     buffered: Option<Box<[u8]>>,
//     done: bool,
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// struct JsSink {
//     shared_state: Arc<Mutex<JsSinkSharedState>>,
//     // Must be a Box because JsSinkFuture requires a lifetime assertion which is
//     // unavailable for wasm_bindgen exports
//     fut: Option<Box<dyn Future<Output = Option<Result<Box<[u8]>, String>>> + Unpin>>,
//     sink: &'static js_sys::Function,
// }

// impl JsSink {
//     fn new(sink: &'static js_sys::Function) -> Self {
//         let shared_state = Arc::new(Mutex::new(JsSinkSharedState {
//             message_available: false,
//             sink_attached: false,
//             message_waker: None,
//             sink_attached_waker: None,
//             ready_to_send_waker: None,
//             buffered: None,
//             done: false,
//             ended: false,
//         }));

//         let js_sink = Self {
//             shared_state,
//             fut: None,
//             sink,
//         };

//         js_sink.fut = Some(Box::new(JsSinkFuture::new(&mut js_sink)));

//         js_sink
//     }
// }

// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// impl JsSink {
//     #[wasm_bindgen]
//     pub async fn next(&mut self) -> Result<JsValue, JsValue> {
//         // lock it somehow
//         let foo = self.fut.unwrap().as_mut().await;
//         self.fut = Some(Box::new(JsSinkFuture::new(self)));

//         to_jsvalue_stream(foo)
//     }
// }

// struct JsSinkFuture<'a> {
//     js_sink: &'a mut JsSink,
// }

// impl<'a> JsSinkFuture<'a> {
//     fn new(js_sink: &'a mut JsSink) -> Self {
//         Self { js_sink }
//     }
// }

// impl Future for JsSinkFuture<'_> {
//     type Output = Option<Result<Box<[u8]>, String>>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let shared_state = self.js_sink.shared_state.lock().unwrap();

//         if shared_state.message_available {
//             shared_state.message_available = false;
//             return Poll::Ready(Some(Ok(shared_state.buffered.unwrap())));
//         }

//         if !shared_state.sink_attached {
//             shared_state.sink_attached_waker.take().unwrap().wake();
//         }

//         shared_state.message_waker = Some(cx.waker().clone());
//         Poll::Pending
//     }
// }

// impl Sink<Box<[u8]>> for JsSink {
//     type Error = String;

//     fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         let shared_state = self.shared_state.lock().unwrap();

//         if shared_state.sink_attached {
//             match shared_state.message_waker {
//                 Some(x) => Poll::Ready(Ok(())),
//                 None => {
//                     shared_state.ready_to_send_waker = Some(cx.waker().clone());
//                     Poll::Pending
//                 }
//             }
//         } else {
//             shared_state.sink_attached = true;

//             JsValue::from(Closure::new(self.next()));
//             self.sink.call1(&JsValue::null(), &self);

//             shared_state.sink_attached_waker = Some(cx.waker().clone());
//             Poll::Pending
//         }
//     }

//     fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String> {
//         todo!()
//         // let shared_state = self.shared_state.lock().unwrap();

//         // let waker = shared_state.message_waker.take().unwrap();

//         // shared_state.buffered.replace(item);
//         // waker.wake();

//         // if self.ended {
//         //     Err(format!("Stream has already ended"))
//         // }
//         // // self.buffered.replace(item);
//         // // self.waker.unwrap().wake();

//         // Ok(())
//     }

//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }
// }
