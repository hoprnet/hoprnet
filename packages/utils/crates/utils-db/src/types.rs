#[cfg(feature = "wasm")]
pub mod wasm {
    use core::task::{Context, Poll};
    use futures_lite::stream::{Stream, StreamExt};
    use pin_project_lite::pin_project;
    use std::pin::Pin;
    use wasm_bindgen_futures::stream::JsStream;

    pin_project! {
        pub struct BinaryStreamWrapper {
            #[pin]
            stream: JsStream,
        }
    }

    impl BinaryStreamWrapper {
        pub fn new(s: JsStream) -> Self {
            Self { stream: s }
        }
    }

    impl Stream for BinaryStreamWrapper {
        type Item = crate::errors::Result<Box<[u8]>>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.stream.poll_next(cx) {
                Poll::Ready(item) => match item {
                    None => Poll::Ready(None),
                    Some(result) => Poll::Ready(Some(
                        result
                            .map(|v| js_sys::Uint8Array::from(v).to_vec().into_boxed_slice())
                            .map_err(|e| {
                                crate::errors::DbError::GenericError(format!("Failed to poll next value: {:?}", e))
                            }),
                    )),
                },
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
