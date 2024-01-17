//! # HOPR Metrics Collection
//!
//! The purpose of the `hopr_metrics` Rust crate is to create a thin Rust WASM-compatible wrapper
//! over the [Prometheus Metrics Rust API](https://docs.rs/prometheus/latest/prometheus/).
//!
//! The reason for making this wrapper is to make it suitable for `wasm-bindgen` bindings to JS/TS. The
//! `prometheus` crate API is not suitable for `wasm-bindgen` as it is. However, this crate can be also used
//! in pure Rust crates (either WASM or non-WASM compatible) and is not HOPR specific.
//!
//! This wrapper merely simplifies the 3 basic Metric Types:
//!
//! - Counter (integer only)
//! - Gauge (floating point only)
//! - Histogram (floating point only)
//!
//! The above 3 types are wrapped using the following structs:
//!
//! - `SimpleCounter`
//! - `MultiCounter`
//! - `SimpleGauge`
//! - `MultiGauge`
//! - `SimpleHistogram`
//! - `MultiHistogram`
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
//! fn main() {
//!     let metric_counter = SimpleCounter::new("test_counter", "Some testing counter").unwrap();
//!
//!     // Counter can be only incremented by integers only
//!     metric_counter.increment_by(10);
//!
//!     let metric_gauge = SimpleGauge::new("test_gauge", "Some testing gauge").unwrap();
//!
//!     // Gauges can be incremented and decrements and support floats
//!     metric_gauge.increment(5.0);
//!     metric_gauge.decrement(3.2);
//!
//!     let metric_histogram = SimpleHistogram::new("test_histogram", "Some testing histogram", vec![1.0, 2.0]).unwrap();
//!
//!     // Histograms can observe floating point values
//!     metric_histogram.observe(10.1);
//!
//!     // ... and also can be used to measure time durations in seconds
//!     let timer = metric_histogram.start_measure();
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//!     metric_histogram.record_measure(timer);
//!
//!     // Multi-metrics are labeled extensions
//!     let metric_counts_per_version = MultiCounter::new("test_multi_counter", "Testing labeled counter", &["version"]).unwrap();
//!
//!     // Tracks counters per different versions
//!     metric_counts_per_version.increment_by(&["1.0.0"], 2);
//!     metric_counts_per_version.increment_by(&["1.0.1"], 1);
//!
//!     // All metrics live in a global state and can be serialized at any time
//!     let gathered_metrics = gather_all_metrics();
//!
//!     // Metrics are in text format and can be exposed using an HTTP API endpoint
//!     println!("{:?}", gathered_metrics);
//! }
//! ```
//!
//! ### Usage in JS/TS
//!
//! Because the crate is WASM compatible, it contains also TS/JS bindings via
//! `wasm-bindgen`.
//! Once a metric is created, it lives in a global registry. Values of all
//! created metrics can be gathered in a serialized text for at any time.
//!
//! See the example below for details.
//!
//! #### Example use in JS/TS
//!
//! ```js
//! const metric_counter = create_counter('test_counter', 'Some testing counter')
//!
//! // Counter can be only incremented by integers only
//! metric_counter.increment_by(10)
//!
//! const metric_gauge = create_counter('test_gauge', 'Some testing gauge')
//!
//! // Gauges can be incremented and decrements and support floats
//! metric_gauge.increment_by(5)
//! metric_gauge.decrement_by(3.2)
//!
//! const metric_histogram = create_histogram('test_histogram', 'Some testing histogram')
//!
//! // Histograms can observe floating point values
//! metric_histogram.observe(10.1)
//!
//! // ... and also can be used to measure time durations in seconds
//! const timer = metric_histogram.start_measure()
//! foo()
//! metric_histogram.record_measure(timer)
//!
//! // Multi-metrics are labeled extensions
//! const metric_countsPerVersion = create_multi_counter('test_multi_counter', 'Testing labeled counter', ['version'])
//!
//! // Tracks counters per different versions
//! metric_countsPerVersion.increment_by('1.0.0', 2)
//! metric_countsPerVersion.increment_by('1.0.1', 1)
//!
//! // All metrics live in a global state and can be serialized at any time
//! let gathered_metrics = gather_all_metrics()
//!
//! // Metrics are in text format and can be exposed using an HTTP API endpoint
//! console.log(gathered_metrics)
//! ```
//!
//! ### Scraping metrics across separate WASM runtimes in HOPRd
//!
//! Each WASM module runs in its own separate runtime and has a private memory space. This has implications for the metric
//! registries used by Prometheus in HOPRd. As a direct consequence of this fact, each WASM runtime has its own global metric
//! registry which has to be scraped separately in order to have all the metrics exposed.
//!
//! Each Rust crate that will become a separate WASM runtime instantiated from the TS in HOPRd, must declare a public WASM-bound
//! function for gathering metrics:
//!
//! ```js
//! #[wasm_bindgen]
//! pub fn my_crate_gather_metrics() -> JsResult<String> {
//!     hopr_metrics::metrics::wasm::gather_all_metrics()
//! }
//! ```
//!
//! and when initializing the crate in TS, `registerMetricsCollector` function must be called
//! passing the `my_crate_gather_metrics` as argument:
//!
//! ```typescript
//! import registerMetricsCollector from '@hoprnet/hopr-utils'
//! import { my_crate_initialize_crate, my_crate_gather_metrics } from '../lib/my_crate.js'
//! my_crate_initialize_crate()
//! registerMetricsCollector(my_crate_gather_metrics)
//! ```

pub mod metrics;
