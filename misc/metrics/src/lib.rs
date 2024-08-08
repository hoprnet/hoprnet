//! # HOPR Metrics Collection
//!
//! The purpose of the `hopr_metrics` Rust crate is to create a thin wrapper
//! over the [Prometheus Metrics Rust API](https://docs.rs/prometheus/latest/prometheus/) or
//! a similar metrics library, should it change in the future.
//!
//! This wrapper merely simplifies the 3 basic Metric Types:
//!
//! - Counter (integer only)
//! - Gauge (floating point only)
//! - Histogram (floating point only)
//!
//! The above 3 types are wrapped using the following structs:
//!
//! - [SimpleCounter](crate::metrics::SimpleCounter)
//! - [MultiCounter](crate::metrics::MultiCounter)
//! - [SimpleGauge](crate::metrics::SimpleGauge)
//! - [MultiGauge](crate::metrics::MultiGauge)
//! - [SimpleHistogram](crate::metrics::SimpleHistogram)
//! - [MultiHistogram](crate::metrics::MultiHistogram)
//!
//! The "simple" types represent a singular named metrics, whereas the "multi" metrics represent a
//! vector extension.
//!
//! The vector extensions basically maintain multiple labelled metrics in a single
//! entity. This makes it possible to have categorized metric values within a single metric, e.g.
//! counter of successful HTTP requests categorized by HTTP method.
//!
//! The metrics are registered within global metrics registry (singleton).
//! Currently, the crate does not support additional individual registries apart from the global one.
//!
//! ### Usage in Rust code
//!
//! When writing pure Rust code that uses this crate, one can use the above structs by directly instantiating them.
//! During their construction, the metric registers itself in the global metrics registry.
//!
//! #### Example use in Rust
//!
//! ```rust
//! use hopr_metrics::metrics::*;
//!
//! let metric_counter = SimpleCounter::new("test_counter", "Some testing counter").unwrap();
//!
//! // Counter can be only incremented by integers only
//! metric_counter.increment_by(10);
//!
//! let metric_gauge = SimpleGauge::new("test_gauge", "Some testing gauge").unwrap();
//!
//! // Gauges can be incremented and decrements and support floats
//! metric_gauge.increment(5.0);
//! metric_gauge.decrement(3.2);
//!
//! let metric_histogram = SimpleHistogram::new("test_histogram", "Some testing histogram", vec![1.0, 2.0]).unwrap();
//!
//! // Histograms can observe floating point values
//! metric_histogram.observe(10.1);
//!
//! // ... and also can be used to measure time durations in seconds
//! let timer = metric_histogram.start_measure();
//! std::thread::sleep(std::time::Duration::from_secs(1));
//! metric_histogram.record_measure(timer);
//!
//! // Multi-metrics are labeled extensions
//! let metric_counts_per_version = MultiCounter::new("test_multi_counter", "Testing labeled counter", &["version"]).unwrap();
//!
//! // Tracks counters per different versions
//! metric_counts_per_version.increment_by(&["1.0.0"], 2);
//! metric_counts_per_version.increment_by(&["1.0.1"], 1);
//!
//! // All metrics live in a global state and can be serialized at any time
//! let gathered_metrics = gather_all_metrics();
//!
//! // Metrics are in text format and can be exposed using an HTTP API endpoint
//! println!("{:?}", gathered_metrics);
//! ```
//!

/// Contains definitions of metric types.
pub mod metrics;
