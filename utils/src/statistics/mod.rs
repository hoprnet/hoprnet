#[cfg(feature = "statistics-types-moving")]
pub mod moving;

#[cfg(feature = "statistics-types-moving")]
pub use moving::{
    exponential::ExponentialMovingAverage,
    simple::{NoSumSMA, SMA, SingleSumSMA},
};

#[cfg(feature = "statistics-types-weighted")]
pub mod weighted;

#[cfg(feature = "statistics-types-weighted")]
pub use weighted::WeightedCollection;

#[cfg(feature = "statistics-types")]
pub mod descriptive;

#[cfg(feature = "statistics-types")]
pub use descriptive::{median, std_dev, variance};
