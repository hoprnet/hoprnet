use core::panic;
use std::{pin::Pin, u8};

use futures::{
    future::{FutureExt, LocalBoxFuture},
    stream::{FusedStream, Stream},
    task::{Context, Poll},
    Future, Sink, SinkExt,
};
use js_sys::{AsyncIterator, Function, Object, Promise, Symbol, Reflect};
use pin_project_lite::pin_project;
use std::task::Waker;
use utils_misc::{async_iterable::wasm::to_jsvalue_stream, ok_or_jserr};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture};

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
        #[pin]
        sink_flushed_fut: Option<LocalBoxFuture<'a, ()>>,
        next_send_chunk: Option<Box<[u8]>>,
        waker: Option<Waker>,
        #[pin]
        resolve: Option<Function>,
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
            next_send_chunk: None,
            waker: None,
            sink_flushed_fut: None,
            resolve: None
        }
    }
}

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

        // let mut this = self.project();
        let mut this = unsafe { std::mem::transmute::<Pin<&mut MyJsStreamStruct<'a>>, Pin<&mut MyJsStreamStruct<'static>>>(self) }.project();

        *this.waker = Some(cx.waker().clone());
    
        if this.sink_flushed_fut.is_none() {
            log("about to call sink");
            this.sink_flushed_fut.set(Some(
                async {
                    log("sink calling code called");

                    let fun2 = move || {
                        log("rs: iterator code called");

                        js_sys::Promise::new(&mut |resolve, reject| {
                            this.resolve.set(Some(resolve));
                            if let Some(waker) = this.waker.take() {
                                log("waking up");
                                waker.wake();
                            }
                        })
                    };

                    let next_chunk_closure: Closure<dyn FnMut() -> Promise> =
                        Closure::new(fun2);
                    let inner_obj = Object::new();
                    
                    Reflect::set(
                        &inner_obj,
                        &"next".into(),
                        next_chunk_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    let obj = Object::new();
                    let iterator_closure: Closure<dyn FnMut() -> Object> =
                        Closure::once(move || inner_obj);
                    Reflect::set(
                        &obj,
                        &Symbol::async_iterator(),
                        // Cast Closure to js_sys::Function
                        iterator_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                    // Resolves once stream is closed
                    this.js_stream.sink(&obj).await;
                }
                .boxed_local(),
            ));

            return match this.sink_flushed_fut.as_pin_mut().unwrap().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(Err("Stream has been closed".into()))
            }
        } else if this.resolve.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String> {
        log("start_send called");

        let mut this = self.project();

        match this.resolve.take() {
            Some(f) => {
                match f.call1(&JsValue::undefined(), &to_jsvalue_stream(Some(Ok(item))).unwrap()) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("{:?}", e).into())
                }
            }
            None => Err("Sink is not yet ready. Please `await` poll_ready first".into())
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        if let Some(fut) = this.sink_flushed_fut.as_mut().as_pin_mut() {
            return match fut.poll(cx) {
                Poll::Ready(_) => Poll::Ready(Ok::<(), Self::Error>(())),
                Poll::Pending => Poll::Pending,
            }
        } 

        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

}

#[wasm_bindgen]
pub async fn foo_bar(stream: MyJsStream) {
    let mut foo = std::mem::ManuallyDrop::new(MyJsStreamStruct::from(&stream));

    // foo.send_all(stream)
    ok_or_jserr!(foo.send(Box::new([0u8, 1u8])).await);
    ok_or_jserr!(foo.send(Box::new([2u8, 3u8])).await);

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
