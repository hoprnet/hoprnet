use core::panic;
use std::{pin::Pin, u8};

use futures::{
    channel::mpsc,
    future::{BoxFuture, FutureExt, LocalBoxFuture},
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, Sink, SinkExt, StreamExt,
};
use js_sys::{AsyncIterator, Function, Object};
use pin_project_lite::pin_project;
use std::mem::ManuallyDrop;
use std::task::Waker;
use utils_misc::{async_iterable::wasm::to_jsvalue_stream, ok_or_jserr};
use wasm_bindgen::{closure, convert::IntoWasmAbi, prelude::*, JsCast};
use wasm_bindgen_futures::{stream::JsStream, JsFuture};

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
        js_stream: &'a MyJsStream,
        // shared_state: Arc<Mutex<MyJsStreamSharedStructSharedState>>,
        #[pin]
        fut: Option<MyJsStreamStructFuture>,
        // fut: Option<LocalBoxFuture<'a, Result<JsValue, JsValue>>>,
        #[pin]
        sink_flushed_fut: Option<LocalBoxFuture<'a, ()>>,
        next_send_chunk: Option<Box<[u8]>>,
        waker: Option<std::task::Waker>,
    }
}

pin_project! {
struct MyJsStreamStructFuture {
    waker: Option<std::task::Waker>,
    // None => no value to send at the moment
    // Some(Some(Box<[u8]>)) => one chunk pending to send
    // Some(None) => stream closed
    next_send_chunk: Option<Option<Result<Box<[u8]>, String>>>,
}
}

trait SomeTrait {
    fn take_item(&mut self, item: Box<[u8]>) -> ();
}

impl MyJsStreamStructFuture {
    fn new() -> MyJsStreamStructFuture {
        MyJsStreamStructFuture {
            waker: None,
            next_send_chunk: None,
        }
    }

    fn take_item(&mut self, item: Box<[u8]>) {
        self.next_send_chunk.insert(Some(Ok(item)));
        self.waker.as_ref().unwrap().wake_by_ref();
    }
}

impl Future for MyJsStreamStructFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.waker.replace(cx.waker().clone());

        // if self.js_stream.
        // let mut this = unsafe { self.get_unchecked_mut() };

        if this.next_send_chunk.is_none() {
            Poll::Pending
        } else {
            Poll::Ready(utils_misc::async_iterable::wasm::to_jsvalue_stream(
                this.next_send_chunk.take().unwrap(),
            ))
        }
        // foo.
        // if this.js_stream.next_chunk.is_none() {
        //     this.js_stream.waker = Some(cx.waker().clone());
        //     Poll::Pending
        // } else {
        //     let to_send = self.js_stream.next_chunk.take().unwrap();
        //     Poll::Ready(to_send)
        // }

        // Poll::Pending
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
            // shared_state: Arc::new(Mutex::new(MyJsStreamSharedStructSharedState {
            fut: None,
            next_send_chunk: None,
            waker: None,
            sink_flushed_fut: None,
            // })),
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

impl<'a> Sink<Box<[u8]>> for MyJsStreamStruct<'a> {
    type Error = String;
    fn poll_ready(
        self: Pin<&mut MyJsStreamStruct<'a>>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        log("poll_ready called");
        let this = self.project();

        // let fut_taken = unsafe { this.fut.get_unchecked_mut() }.as_mut();

        // let shared_state = self.shared_state.lock().unwrap();
        // let this = ManuallyDrop::new(self.project());
        // let this = Pin::new(self);
        // let js_stream_ptr: &'a MyJsStream = this.js_stream.get_mut();
        // let foo = this.fut.as_mut();
        // let fut_ptr: &mut Option<MyJsStreamStructFuture> = unsafe { this.fut.get_unchecked_mut() };

        // let fut_ptr: &'a mut Option<MyJsStreamStructFuture> =
        // unsafe { this.fut.get_unchecked_mut() };

        if this.sink_flushed_fut.is_none() {
            Pin::new(unsafe { this.sink_flushed_fut.get_unchecked_mut() }).set(Some(
                async {
                    // fut_ptr;
                    log("sink calling code called");
                    // this.fut.take();
                    // fut_ptr;
                    // let foo = this.fut;

                    // let fun = move || {
                    //     // if this.waker.is_some() {
                    //     //     this.waker.take().unwrap().wake();
                    //     // }
                    //     // let current = unsafe { this.fut.get_unchecked_mut() }.take();
                    //     // this.fut.set(Some(MyJsStreamStructFuture::new()));
                    //     // let current = fut_ptr.take();
                    //     wasm_bindgen_futures::future_to_promise(fut_taken.unwrap())

                    //     // js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
                    //     // js_sys::Reflect::set(&obj, &"value".into(), &JsValue::FALSE).unwrap();
                    //     // obj
                    // };

                    // let foo = self.next();

                    let fun2 = || {
                        // create static Wrapper and implement Stream interface
                        let fut = MyJsStreamStructFuture::new();
                        wasm_bindgen_futures::future_to_promise(async {
                            // let fut = Pin::new(unsafe { this.fut.get_unchecked_mut() }).unwrap();
                            fut.await
                        });

                        // println!("{:?}", fut_taken);
                        js_sys::Promise::resolve(&JsValue::from(""))
                    };

                    let next_chunk_closure: Closure<dyn FnMut() -> js_sys::Promise> =
                        Closure::once(fun2);
                    let inner_obj = js_sys::Object::new();
                    js_sys::Reflect::set(
                        &inner_obj,
                        &"next".into(),
                        next_chunk_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    let obj = js_sys::Object::new();
                    let iterator_closure: Closure<dyn FnMut() -> js_sys::Object> =
                        Closure::once(move || inner_obj);
                    js_sys::Reflect::set(
                        &obj,
                        &js_sys::Symbol::async_iterator(),
                        // Cast Closure to js_sys::Function
                        iterator_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    this.js_stream.sink(&JsValue::from("")).await;
                }
                .boxed_local(),
            ))
        }

        // this.sink_flushed_fut;

        // if this.sink_flushed_fut.is_none() {
        //     log("calling sink");
        //     *this.sink_flushed_fut = Some(Box::pin(async {
        //         // fut_ptr;
        //         log("sink calling code called");
        //         // this.fut.take();
        //         // fut_ptr;
        //         // let foo = this.fut;
        //         let next_chunk_closure: Closure<dyn FnMut() -> js_sys::Promise> =
        //             Closure::new(move || {
        //                 // if this.waker.is_some() {
        //                 //     this.waker.take().unwrap().wake();
        //                 // }
        //                 // let current = this.fut.take();
        //                 // let current = fut_ptr.take();
        //                 wasm_bindgen_futures::future_to_promise(
        //                     Some(MyJsStreamStructFuture::new()).unwrap(),
        //                 )
        //                 // js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
        //                 // js_sys::Reflect::set(&obj, &"value".into(), &JsValue::FALSE).unwrap();
        //                 // obj
        //             });
        //         let inner_obj = js_sys::Object::new();
        //         js_sys::Reflect::set(
        //             &inner_obj,
        //             &"next".into(),
        //             next_chunk_closure.as_ref().unchecked_ref(),
        //         )
        //         .unwrap();

        //         let obj = js_sys::Object::new();
        //         let iterator_closure: Closure<dyn FnMut() -> js_sys::Object> =
        //             Closure::once(move || inner_obj);
        //         js_sys::Reflect::set(
        //             &obj,
        //             &js_sys::Symbol::async_iterator(),
        //             // Cast Closure to js_sys::Function
        //             iterator_closure.as_ref().unchecked_ref(),
        //         )
        //         .unwrap();

        //         // js_stream_ptr.sink(&obj).await;

        //         log("awaited");
        //     }));
        // }

        // let this = self.project();

        // if this.fut.is_none() {
        //     // this.waker.replace(cx.waker().clone());
        //     return Poll::Pending;
        // }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
        log("poll_ready called");

        let this = self.project();

        // let foo = this.fut;
        // .this
        // .fut
        // .as_mut()
        // .unwrap()
        // .take(item);

        Ok(())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        // let foo = this.sink_flushed_fut.as_mut().unwrap().poll_unpin(cx)
        // if let Some(fut) = this.sink_flushed_fut.as_mut().as_pin_mut() {
        //     match fut.poll_unpin(cx) {
        //         Poll::Ready(_) => Poll::Ready(Ok::<(), Self::Error>(())),
        //         Poll::Pending => Poll::Pending,
        //     }
        // }

        Poll::Pending
    }
}

#[wasm_bindgen]
pub async fn foo_bar(stream: MyJsStream) {
    let mut foo = ManuallyDrop::new(MyJsStreamStruct::from(&stream));

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
