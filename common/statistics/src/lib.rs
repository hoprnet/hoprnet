#[cfg(feature = "moving")]
pub mod moving;

#[cfg(feature = "moving")]
pub use moving::{
    exponential::ExponentialMovingAverage,
    simple::{NoSumSMA, SMA, SingleSumSMA},
};
