/// Converts from duplex JS stream to Rust `futures::Stream` / `futures::Sink`
use crate::{async_iterable::wasm::to_jsvalue_iterator, traits::DuplexStream};
use core::{panic, pin::Pin, task::Waker};
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

// TODO: make this thread-safe using an Arc
pin_project! {
    /// Holds a Javascript Streaming Iterable object and
    /// implements Rust `futures::Sink` and `futures::Stream` trait
    #[derive(Debug)]
    struct StreamingIterableInner {
        // stream iterator
        iter: RefCell<Option<AsyncIterator>>,
        // stream done
        stream_done: RefCell< bool>,
        // next stream chunk
        next: RefCell<Option<JsFuture>>,
        // sink closed promise
        sink_close_future: RefCell<Option<JsFuture>>,
        // signals that sink `Sink::poll_close` future can proceed
        close_waker: RefCell<Option<Waker>>,
        // signals that sink `Sink::poll_ready` future can proceed
        waker: RefCell< Option<Waker>>,
        // takes sink chunks
        resolve: RefCell< Option<Function>>,
        // true once `poll_close` has been called
        sink_done: RefCell< bool>,
        // the Javascript StreamingIterable object
        js_stream: JsStreamingIterable,
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct StreamingIterable {
        inner: Rc<StreamingIterableInner>
    }
}

// Turn this into an arc
impl From<JsStreamingIterable> for StreamingIterable {
    fn from(stream: JsStreamingIterable) -> Self {
        Self {
            inner: Rc::new(StreamingIterableInner {
                js_stream: stream,
                stream_done: RefCell::new(false),
                next: RefCell::new(None),
                iter: RefCell::new(None),
                waker: RefCell::new(None),
                close_waker: RefCell::new(None),
                sink_close_future: RefCell::new(None),
                resolve: RefCell::new(None),
                sink_done: RefCell::new(false),
            }),
        }
    }
}

impl Stream for StreamingIterable {
    type Item = Result<Box<[u8]>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.inner.stream_done.borrow() {
            return Poll::Ready(None);
        }

        let mut stream_done_mut = this.inner.stream_done.borrow_mut();
        let mut iter_mut = this.inner.iter.borrow_mut();

        if iter_mut.is_none() {
            let async_sym = Symbol::async_iterator();

            let initial = this.inner.js_stream.source();
            let async_iter_fn = match Reflect::get(&initial, async_sym.as_ref()) {
                Ok(x) => x,
                Err(_) => return Poll::Ready(None),
            };

            let async_iter_fn: Function = match async_iter_fn.dyn_into() {
                Ok(fun) => fun,
                Err(e) => {
                    error!("error while dynamic conversion to function {:?}", e);
                    *stream_done_mut = true;
                    return Poll::Ready(None);
                }
            };

            let async_it: AsyncIterator = match async_iter_fn.call0(&initial).unwrap().dyn_into() {
                Ok(x) => x,
                Err(e) => {
                    error!("error while dynamic conversion to async iterator {:?}", e);
                    *stream_done_mut = true;
                    return Poll::Ready(None);
                }
            };

            *iter_mut = Some(async_it);
        }

        let mut next_mut = this.inner.next.borrow_mut();

        let future = match next_mut.as_mut() {
            Some(val) => val,
            None => match iter_mut.as_ref().unwrap().next().map(JsFuture::from) {
                Ok(val) => {
                    *next_mut = Some(val);
                    next_mut.as_mut().unwrap()
                }
                Err(e) => {
                    *stream_done_mut = true;
                    return Poll::Ready(Some(Err(format!("{:?}", e))));
                }
            },
        };

        match Pin::new(future).poll(cx) {
            Poll::Ready(res) => match res {
                Ok(iter_next) => {
                    let next = iter_next.unchecked_into::<Uint8ArrayIteratorNext>();
                    if next.done() {
                        *stream_done_mut = true;
                        Poll::Ready(None)
                    } else {
                        next_mut.take();

                        Poll::Ready(Some(Ok(next.value())))
                    }
                }
                Err(e) => {
                    *stream_done_mut = true;
                    Poll::Ready(Some(Err(format!("{:?}", e))))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FusedStream for StreamingIterable {
    fn is_terminated(&self) -> bool {
        *self.inner.stream_done.borrow()
    }
}

impl Sink<Box<[u8]>> for StreamingIterable {
    type Error = String;

    fn poll_ready(self: Pin<&mut StreamingIterable>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        {
            *this.inner.waker.borrow_mut() = Some(cx.waker().clone());
            // drop mutable borrow here
        }

        let mut sink_done_mut = this.inner.sink_done.borrow_mut();

        if *sink_done_mut {
            return Poll::Ready(Err("Cannot send any data. Stream has been closed".into()));
        }

        if this.inner.resolve.borrow().is_some() {
            return Poll::Ready(Ok(()));
        }

        let mut sink_close_future_mut = this.inner.sink_close_future.borrow_mut();

        if sink_close_future_mut.is_none() {
            let self_clone = Rc::clone(this.inner);
            let iterator_cb = Closure::<dyn Fn() -> Promise>::new(move || {
                Promise::new(&mut |resolve, _reject| {
                    // TODO: use borrow_mut()
                    *self_clone.resolve.borrow_mut() = Some(resolve);
                    if let Some(waker) = self_clone.close_waker.borrow_mut().take() {
                        waker.wake();
                    } else if let Some(waker) = self_clone.waker.borrow_mut().take() {
                        waker.wake();
                    }
                })
            });

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

            let promise = match this.inner.js_stream.sink(&iterable_obj) {
                Ok(x) => {
                    let promise = x.unchecked_into::<Promise>();

                    JsFuture::from(promise)
                }
                Err(e) => {
                    error!("error while calling sink {:?}", e);
                    todo!();
                }
            };

            *sink_close_future_mut = Some(promise);

            return match Pin::new(&mut sink_close_future_mut.as_mut().unwrap()).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(res) => Poll::Ready(match res {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        *sink_done_mut = true;
                        Err(format!("Stream closed due to error {:?}", e).into())
                    }
                }),
            };
        }

        Poll::Pending
    }

    fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), String> {
        let this = self.project();

        match this.inner.resolve.borrow_mut().take() {
            Some(f) => match f.call1(&JsValue::undefined(), &to_jsvalue_iterator(Some(item))) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("error while calling resolve function {:?}", e);
                    Err(format!("error while calling resolve function {:?}", e).into())
                }
            },
            None => Err("Sink is not yet ready. Please call and `await` poll_ready first".into()),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();

        let mut sink_close_future_mut = this.inner.sink_close_future.borrow_mut();

        if sink_close_future_mut.is_none() {
            return Poll::Ready(Err("Uninitialized. Please call and `await` poll_ready first.".into()));
        }

        *this.inner.close_waker.borrow_mut() = Some(cx.waker().clone());

        let mut sink_done_mut = this.inner.sink_done.borrow_mut();

        if !*sink_done_mut {
            match this.inner.resolve.take() {
                None => return Poll::Pending,
                Some(f) => match f.call1(&JsValue::undefined(), &to_jsvalue_iterator(None)) {
                    Ok(_) => {
                        *sink_done_mut = true;
                    }
                    Err(e) => {
                        // We cannot close stream due to some issue in Javascript,
                        // so mark stream closed to prevent subsequent calls
                        *sink_done_mut = true;
                        return Poll::Ready(Err(format!("{:?}", e).into()));
                    }
                },
            }
        }

        if let Some(fut) = sink_close_future_mut.as_mut() {
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
