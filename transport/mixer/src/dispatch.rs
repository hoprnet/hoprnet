//! Runtime dispatch over the configured mixer engine.
//!
//! [`create`] instantiates the engine named by [`MixerConfig::mixer_type`] and wraps its sender
//! and receiver in enums that forward `Sink`/`Stream` to the active variant, so callers stay
//! agnostic to which engine runs. The `match` per poll is negligible against packet processing.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Sink, Stream};

use crate::{
    config::{MixerConfig, MixerType},
    error::SenderError,
};

/// Sender over whichever engine [`create`] selected.
pub enum AnySender<T> {
    #[cfg(feature = "uniform-channel")]
    Uniform(crate::channel::Sender<T>),
    #[cfg(feature = "poisson")]
    Poisson(crate::poisson::Sender<T>),
    #[cfg(feature = "poisson-shared")]
    PoissonShared(crate::poisson_shared::Sender<T>),
}

impl<T> Clone for AnySender<T> {
    fn clone(&self) -> Self {
        match self {
            #[cfg(feature = "uniform-channel")]
            AnySender::Uniform(s) => AnySender::Uniform(s.clone()),
            #[cfg(feature = "poisson")]
            AnySender::Poisson(s) => AnySender::Poisson(s.clone()),
            #[cfg(feature = "poisson-shared")]
            AnySender::PoissonShared(s) => AnySender::PoissonShared(s.clone()),
        }
    }
}

impl<T> Sink<T> for AnySender<T> {
    type Error = SenderError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.get_mut() {
            #[cfg(feature = "uniform-channel")]
            AnySender::Uniform(s) => Pin::new(s).poll_ready(cx),
            #[cfg(feature = "poisson")]
            AnySender::Poisson(s) => Pin::new(s).poll_ready(cx),
            #[cfg(feature = "poisson-shared")]
            AnySender::PoissonShared(s) => Pin::new(s).poll_ready(cx),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        match self.get_mut() {
            #[cfg(feature = "uniform-channel")]
            AnySender::Uniform(s) => Pin::new(s).start_send(item),
            #[cfg(feature = "poisson")]
            AnySender::Poisson(s) => Pin::new(s).start_send(item),
            #[cfg(feature = "poisson-shared")]
            AnySender::PoissonShared(s) => Pin::new(s).start_send(item),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.get_mut() {
            #[cfg(feature = "uniform-channel")]
            AnySender::Uniform(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "poisson")]
            AnySender::Poisson(s) => Pin::new(s).poll_flush(cx),
            #[cfg(feature = "poisson-shared")]
            AnySender::PoissonShared(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.get_mut() {
            #[cfg(feature = "uniform-channel")]
            AnySender::Uniform(s) => Pin::new(s).poll_close(cx),
            #[cfg(feature = "poisson")]
            AnySender::Poisson(s) => Pin::new(s).poll_close(cx),
            #[cfg(feature = "poisson-shared")]
            AnySender::PoissonShared(s) => Pin::new(s).poll_close(cx),
        }
    }
}

/// Receiver over whichever engine [`create`] selected.
pub enum AnyReceiver<T> {
    #[cfg(feature = "uniform-channel")]
    Uniform(crate::channel::Receiver<T>),
    #[cfg(feature = "poisson")]
    Poisson(crate::poisson::Receiver<T>),
    #[cfg(feature = "poisson-shared")]
    PoissonShared(crate::poisson_shared::Receiver<T>),
}

impl<T: Unpin> Stream for AnyReceiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            #[cfg(feature = "uniform-channel")]
            AnyReceiver::Uniform(r) => Pin::new(r).poll_next(cx),
            #[cfg(feature = "poisson")]
            AnyReceiver::Poisson(r) => Pin::new(r).poll_next(cx),
            #[cfg(feature = "poisson-shared")]
            AnyReceiver::PoissonShared(r) => Pin::new(r).poll_next(cx),
        }
    }
}

/// Instantiate the mixer engine selected by `cfg.mixer_type`, returning an engine-agnostic
/// sender/receiver pair.
pub fn create<T: Send + Unpin + 'static>(cfg: MixerConfig) -> (AnySender<T>, AnyReceiver<T>) {
    match cfg.mixer_type {
        #[cfg(feature = "uniform-channel")]
        MixerType::Uniform => {
            let (tx, rx) = crate::channel::channel(cfg);
            (AnySender::Uniform(tx), AnyReceiver::Uniform(rx))
        }
        #[cfg(feature = "poisson")]
        MixerType::Poisson(_) => {
            let (tx, rx) = crate::poisson::poisson_channel(cfg);
            (AnySender::Poisson(tx), AnyReceiver::Poisson(rx))
        }
        #[cfg(feature = "poisson-shared")]
        MixerType::PoissonShared(_) => {
            let (tx, rx) = crate::poisson_shared::poisson_shared_channel(cfg);
            (AnySender::PoissonShared(tx), AnyReceiver::PoissonShared(rx))
        }
    }
}
