use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportMixerError {}

pub type Result<T> = std::result::Result<T, TransportMixerError>;
