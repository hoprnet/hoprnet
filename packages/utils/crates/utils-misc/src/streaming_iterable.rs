/// Converts from duplex JS stream to Rust `futures::Stream` / `futures::Sink`
use crate::{async_iterable::wasm::to_jsvalue_iterator, traits::DuplexStream};
use core::{pin::Pin, task::Waker};
use futures::{
    stream::FusedStream,
    task::{Context, Poll},
    Future, Sink, Stream,
};
use js_sys::{AsyncIterator, Function, Object, Promise, Reflect, Symbol};
use pin_project_lite::pin_project;
use std::{cell::RefCell, rc::Rc};
use utils_log::{error, info};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Typing of the Javascrip duplex stream
#[wasm_bindgen(typescript_custom_section)]
const IStream: &'static str = r#"
interface IStream {
    sink(source: AsyncIterable<Uint8Array>): Promise<void>;
    source: AsyncIterable<Uint8Array>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(is_type_of = AsyncIterable::looks_like_async_iterable, typescript_type = "AsyncIterable<Uint8Array>")]
    pub type AsyncIterable;
}

impl AsyncIterable {
    fn looks_like_async_iterable(it: &JsValue) -> bool {
        if !it.is_object() {
            return false;
        }

        let async_sym = Symbol::async_iterator();
        let async_iter_fn = match Reflect::get(it, async_sym.as_ref()) {
            Ok(f) => f,
            Err(_) => return false,
        };

        async_iter_fn.is_function()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = IStream)]
    #[derive(Clone, Debug)]
    pub type JsStreamingIterable;

    #[wasm_bindgen(structural, method, catch)]
    pub fn sink(this: &JsStreamingIterable, stream: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(structural, method, getter)]
    pub fn source(this: &JsStreamingIterable) -> JsValue;
}

/// Special-purpose version of js_sys::IteratorNext for Uint8Arrays
#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type Uint8ArrayIteratorNext;

    #[wasm_bindgen(method, getter, structural)]
    pub fn done(this: &Uint8ArrayIteratorNext) -> bool;

    #[wasm_bindgen(method, getter, structural)]
    pub fn value(this: &Uint8ArrayIteratorNext) -> Box<[u8]>;
}

#[derive(Debug)]
pub struct CloneStuff {
    // signals that sink `Sink::poll_close` future can proceed
    close_waker: Option<Waker>,
    // signals that sink `Sink::poll_ready` future can proceed
    waker: Option<Waker>,
    // takes sink chunks
    resolve: Option<Function>,
}

// TODO: make this thread-safe using an Arc
pin_project! {
/// Holds a Javascript Streaming Iterable object and
/// implements Rust `futures::Sink` and `futures::Stream` trait
    #[derive(Debug)]
    pub struct StreamingIterable {
        // stream iterator
        iter: Option<AsyncIterator>,
        // stream done
        stream_done: bool,
        // next stream chunk
        next: Option<JsFuture>,
        // sink closed promise
        sink_close_future: Option<JsFuture>,
        // signals that sink `Sink::poll_close` future can proceed
        clone_stuff: Rc<RefCell<CloneStuff>>,
        // true once `poll_close` has been called
        sink_done: bool,
        // the Javascript StreamingIterable object
        js_stream: JsStreamingIterable,
    }
}

// Turn this into an arc
impl From<JsStreamingIterable> for StreamingIterable {
    fn from(stream: JsStreamingIterable) -> Self {
        Self {
            js_stream: stream,
            stream_done: false,
            next: None,
            iter: None,
            clone_stuff: Rc::new(RefCell::new(CloneStuff {
                waker: None,
                close_waker: None,
                resolve: None,
            })),
            sink_close_future: None,
            sink_done: false,
        }
    }
}

impl Stream for StreamingIterable {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.stream_done {
            return Poll::Ready(None);
        }

        if this.iter.is_none() {
            let async_sym = Symbol::async_iterator();

            let initial = this.js_stream.source();
            let async_iter_fn = match Reflect::get(&initial, async_sym.as_ref()) {
                Ok(x) => x,
                Err(_) => return Poll::Ready(None),
            };

            let async_iter_fn: Function = match async_iter_fn.dyn_into() {
                Ok(fun) => fun,
                Err(e) => {
                    error!("error while dynamic conversion to function {:?}", e);
                    *this.stream_done = true;
                    return Poll::Ready(None);
                }
            };

            let async_it: AsyncIterator = match async_iter_fn.call0(&initial).unwrap().dyn_into() {
                Ok(x) => x,
                Err(e) => {
                    error!("error while dynamic conversion to async iterator {:?}", e);
                    *this.stream_done = true;
                    return Poll::Ready(None);
                }
            };

            *this.iter = Some(async_it);
        }

        let future = match this.next.as_mut() {
            Some(val) => val,
            None => match this.iter.as_mut().unwrap().next().map(JsFuture::from) {
                Ok(val) => {
                    *this.next = Some(val);
                    this.next.as_mut().unwrap()
                }
                Err(e) => {
                    *this.stream_done = true;
                    return Poll::Ready(Some(Err(format!("{:?}", e))));
                }
            },
        };

        match Pin::new(future).poll(cx) {
            Poll::Ready(res) => match res {
                Ok(iter_next) => {
                    let next = iter_next.unchecked_into::<Uint8ArrayIteratorNext>();
                    if next.done() {
                        *this.stream_done = true;
                        Poll::Ready(None)
                    } else {
                        this.next.take();

                        Poll::Ready(Some(Ok(next.value())))
                    }
                }
                Err(e) => {
                    *this.stream_done = true;
                    Poll::Ready(Some(Err(format!("{:?}", e))))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FusedStream for StreamingIterable {
    fn is_terminated(&self) -> bool {
        self.stream_done
    }
}

impl Sink<Box<[u8]>> for StreamingIterable {
    type Error = String;

    fn poll_ready(self: Pin<&mut StreamingIterable>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        this.clone_stuff.borrow_mut().waker.insert(cx.waker().clone());

        if *this.sink_done {
            return Poll::Ready(Err("Cannot send any data. Stream has been closed".into()));
        }

        if this.clone_stuff.borrow().resolve.is_some() {
            return Poll::Ready(Ok(()));
        }

        if this.sink_close_future.is_none() {
            let iterator_cb = {
                let self_clone = Rc::clone(this.clone_stuff);
                Closure::<dyn Fn() -> Promise>::new(move || {
                    Promise::new(&mut |resolve, _reject| {
                        let mut clone = self_clone.borrow_mut();

                        clone.resolve = Some(resolve);
                        if let Some(waker) = clone.close_waker.take() {
                            waker.wake();
                        } else if let Some(waker) = clone.waker.take() {
                            waker.wake();
                        }
                    })
                })
            };

            let iterator_obj = Object::new();

            // {
            //    next(): Promise<IteratorResult> {
            //      // ... function body
            //    }
            // }
            Reflect::set(&iterator_obj, &"next".into(), iterator_cb.as_ref().unchecked_ref()).unwrap();

            // Release closure to JS garbage collector
            iterator_cb.forget();

            let iterable_fn: Closure<dyn FnMut() -> Object> = Closure::once(move || iterator_obj);

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
                iterable_fn.as_ref().unchecked_ref(),
            )
            .unwrap();

            // Release closure to JS garbage collector
            iterable_fn.forget();

            let promise = match this.js_stream.sink(&iterable_obj) {
                Ok(x) => {
                    let promise = x.unchecked_into::<Promise>();

                    JsFuture::from(promise)
                }
                Err(e) => {
                    error!("error while calling sink {:?}", e);
                    todo!();
                }
            };

            *this.sink_close_future = Some(promise);

            return match Pin::new(&mut this.sink_close_future.as_mut().unwrap()).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(res) => Poll::Ready(match res {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        *this.sink_done = true;
                        Err(format!("Stream closed due to error {:?}", e).into())
                    }
                }),
            };
        }

        Poll::Pending
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String> {
        let this = self.project();

        let resolve = {
            match this.clone_stuff.borrow_mut().resolve.take() {
                Some(f) => f,
                None => return Err("Sink is not yet ready. Please call and `await` poll_ready first".into()),
            }
        };

        match resolve.call1(&JsValue::undefined(), &to_jsvalue_iterator(Some(item))) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("error while calling resolve function {:?}", e);
                Err(format!("error while calling resolve function {:?}", e).into())
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        if this.sink_close_future.is_none() {
            return Poll::Ready(Err("Uninitialized. Please call and `await` poll_ready first.".into()));
        }

        this.clone_stuff.borrow_mut().close_waker.insert(cx.waker().clone());

        if !*this.sink_done {
            let resolve = {
                match this.clone_stuff.borrow_mut().resolve.take() {
                    Some(f) => f,
                    None => return Poll::Pending,
                }
            };

            match resolve.call1(&JsValue::undefined(), &to_jsvalue_iterator(None)) {
                Ok(_) => {
                    *this.sink_done = true;
                }
                Err(e) => {
                    // We cannot close stream due to some issue in Javascript,
                    // so mark stream closed to prevent subsequent calls
                    *this.sink_done = true;
                    return Poll::Ready(Err(format!("{:?}", e).into()));
                }
            }
        }

        if let Some(fut) = this.sink_close_future.as_mut() {
            return match Pin::new(fut).poll(cx) {
                Poll::Ready(_) => Poll::Ready(Ok(())),
                Poll::Pending => Poll::Pending,
            };
        }

        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl DuplexStream for StreamingIterable {}
