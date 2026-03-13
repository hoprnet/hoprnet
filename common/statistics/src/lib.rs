#[cfg(feature = "moving")]
pub mod moving;

#[cfg(feature = "moving")]
pub use moving::{
    exponential::ExponentialMovingAverage,
    simple::{NoSumSMA, SMA, SingleSumSMA},
};

#[cfg(feature = "weighted")]
pub mod weighted;

#[cfg(feature = "weighted")]
pub use weighted::WeightedCollection;
