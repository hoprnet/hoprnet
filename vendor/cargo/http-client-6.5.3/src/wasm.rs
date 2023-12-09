//! http-client implementation for fetch

use std::convert::{Infallible, TryFrom};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::prelude::*;

use crate::Config;

use super::{http_types::Headers, Body, Error, HttpClient, Request, Response};

/// WebAssembly HTTP Client.
#[derive(Debug)]
pub struct WasmClient {
    config: Config,
}

impl WasmClient {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Default for WasmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for WasmClient {
    fn send<'a, 'async_trait>(
        &'a self,
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + 'async_trait>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        let config = self.config.clone();

        InnerFuture::new(async move {
            let req: fetch::Request = fetch::Request::new(req).await?;
            let conn = req.send();
            let mut res = if let Some(timeout) = config.timeout {
                async_std::future::timeout(timeout, conn).await??
            } else {
                conn.await?
            };

            let body = res.body_bytes();
            let mut response =
                Response::new(http_types::StatusCode::try_from(res.status()).unwrap());
            response.set_body(Body::from(body));
            for (name, value) in res.headers() {
                let name: http_types::headers::HeaderName = name.parse().unwrap();
                response.append_header(&name, value);
            }

            Ok(response)
        })
    }

    /// Override the existing configuration with new configuration.
    ///
    /// Config options may not impact existing connections.
    fn set_config(&mut self, config: Config) -> http_types::Result<()> {
        self.config = config;

        Ok(())
    }

    /// Get the current configuration.
    fn config(&self) -> &Config {
        &self.config
    }
}

impl TryFrom<Config> for WasmClient {
    type Error = Infallible;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        Ok(Self { config })
    }
}

struct InnerFuture {
    fut: Pin<Box<dyn Future<Output = Result<Response, Error>> + 'static>>,
}

impl InnerFuture {
    fn new<F: Future<Output = Result<Response, Error>> + 'static>(fut: F) -> Pin<Box<Self>> {
        Box::pin(Self { fut: Box::pin(fut) })
    }
}

// This is safe because WASM doesn't have threads yet. Once WASM supports threads we should use a
// thread to park the blocking implementation until it's been completed.
unsafe impl Send for InnerFuture {}

impl Future for InnerFuture {
    type Output = Result<Response, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is safe because we're only using this future as a pass-through for the inner
        // future, in order to implement `Send`. If it's safe to poll the inner future, it's safe
        // to proxy it too.
        unsafe { Pin::new_unchecked(&mut self.fut).poll(cx) }
    }
}

mod fetch {
    use js_sys::{Array, ArrayBuffer, Reflect, Uint8Array};
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{RequestInit, Window, WorkerGlobalScope};

    use std::iter::{IntoIterator, Iterator};
    use std::pin::Pin;

    use http_types::StatusCode;

    use crate::Error;

    enum WindowOrWorker {
        Window(Window),
        Worker(WorkerGlobalScope),
    }

    impl WindowOrWorker {
        fn new() -> Self {
            #[wasm_bindgen]
            extern "C" {
                type Global;

                #[wasm_bindgen(method, getter, js_name = Window)]
                fn window(this: &Global) -> JsValue;

                #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
                fn worker(this: &Global) -> JsValue;
            }

            let global: Global = js_sys::global().unchecked_into();

            if !global.window().is_undefined() {
                Self::Window(global.unchecked_into())
            } else if !global.worker().is_undefined() {
                Self::Worker(global.unchecked_into())
            } else {
                panic!("Only supported in a browser or web worker");
            }
        }
    }

    /// Create a new fetch request.

    /// An HTTP Fetch Request.
    pub(crate) struct Request {
        request: web_sys::Request,
        /// This field stores the body of the request to ensure it stays allocated as long as the request needs it.
        #[allow(dead_code)]
        body_buf: Pin<Vec<u8>>,
    }

    impl Request {
        /// Create a new instance.
        pub(crate) async fn new(mut req: super::Request) -> Result<Self, Error> {
            // create a fetch request initaliser
            let mut init = RequestInit::new();

            // set the fetch method
            init.method(req.method().as_ref());

            let uri = req.url().to_string();
            let body = req.take_body();

            // convert the body into a uint8 array
            // needs to be pinned and retained inside the Request because the Uint8Array passed to
            // js is just a portal into WASM linear memory, and if the underlying data is moved the
            // js ref will become silently invalid
            let body_buf = body.into_bytes().await.map_err(|_| {
                Error::from_str(StatusCode::BadRequest, "could not read body into a buffer")
            })?;
            let body_pinned = Pin::new(body_buf);
            if body_pinned.len() > 0 {
                let uint_8_array = unsafe { js_sys::Uint8Array::view(&body_pinned) };
                init.body(Some(&uint_8_array));
            }

            let request = web_sys::Request::new_with_str_and_init(&uri, &init).map_err(|e| {
                Error::from_str(
                    StatusCode::BadRequest,
                    format!("failed to create request: {:?}", e),
                )
            })?;

            // add any fetch headers
            let headers: &mut super::Headers = req.as_mut();
            for (name, value) in headers.iter() {
                let name = name.as_str();
                let value = value.as_str();

                request.headers().set(name, value).map_err(|_| {
                    Error::from_str(
                        StatusCode::BadRequest,
                        format!("could not add header: {} = {}", name, value),
                    )
                })?;
            }

            Ok(Self {
                request,
                body_buf: body_pinned,
            })
        }

        /// Submit a request
        // TODO(yoshuawuyts): turn this into a `Future` impl on `Request` instead.
        pub(crate) async fn send(self) -> Result<Response, Error> {
            // Send the request.
            let scope = WindowOrWorker::new();
            let promise = match scope {
                WindowOrWorker::Window(window) => window.fetch_with_request(&self.request),
                WindowOrWorker::Worker(worker) => worker.fetch_with_request(&self.request),
            };
            let resp = JsFuture::from(promise)
                .await
                .map_err(|e| Error::from_str(StatusCode::BadRequest, format!("{:?}", e)))?;

            debug_assert!(resp.is_instance_of::<web_sys::Response>());
            let res: web_sys::Response = resp.dyn_into().unwrap();

            // Get the response body.
            let promise = res.array_buffer().unwrap();
            let resp = JsFuture::from(promise).await.unwrap();
            debug_assert!(resp.is_instance_of::<js_sys::ArrayBuffer>());
            let buf: ArrayBuffer = resp.dyn_into().unwrap();
            let slice = Uint8Array::new(&buf);
            let mut body: Vec<u8> = vec![0; slice.length() as usize];
            slice.copy_to(&mut body);

            Ok(Response::new(res, body))
        }
    }

    /// An HTTP Fetch Response.
    pub(crate) struct Response {
        res: web_sys::Response,
        body: Option<Vec<u8>>,
    }

    impl Response {
        fn new(res: web_sys::Response, body: Vec<u8>) -> Self {
            Self {
                res,
                body: Some(body),
            }
        }

        /// Access the HTTP headers.
        pub(crate) fn headers(&self) -> Headers {
            Headers {
                headers: self.res.headers(),
            }
        }

        /// Get the request body as a byte vector.
        ///
        /// Returns an empty vector if the body has already been consumed.
        pub(crate) fn body_bytes(&mut self) -> Vec<u8> {
            self.body.take().unwrap_or_else(|| vec![])
        }

        /// Get the HTTP return status code.
        pub(crate) fn status(&self) -> u16 {
            self.res.status()
        }
    }

    /// HTTP Headers.
    pub(crate) struct Headers {
        headers: web_sys::Headers,
    }

    impl IntoIterator for Headers {
        type Item = (String, String);
        type IntoIter = HeadersIter;

        fn into_iter(self) -> Self::IntoIter {
            HeadersIter {
                iter: js_sys::try_iter(&self.headers).unwrap().unwrap(),
            }
        }
    }

    /// HTTP Headers Iterator.
    pub(crate) struct HeadersIter {
        iter: js_sys::IntoIter,
    }

    impl Iterator for HeadersIter {
        type Item = (String, String);

        fn next(&mut self) -> Option<Self::Item> {
            let pair = self.iter.next()?;

            let array: Array = pair.unwrap().into();
            let vals = array.values();

            let prop = String::from("value").into();
            let key = Reflect::get(&vals.next().unwrap(), &prop).unwrap();
            let value = Reflect::get(&vals.next().unwrap(), &prop).unwrap();

            Some((
                key.as_string().to_owned().unwrap(),
                value.as_string().to_owned().unwrap(),
            ))
        }
    }
}
