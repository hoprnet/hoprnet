HOPR Metrics Collection
===

The purpose of the `utils-metrics` Rust crate is create a thin Rust WASM-compatible wrapper 
over the [Prometheus Metrics Rust API](https://docs.rs/prometheus/latest/prometheus/).

This wrapper merely simplifies the 3 basic Metric Types:

- Integer Counter
- Float Gauge
- Float Histogram

The above 3 types are wrapped using the following classes:

- `SimpleCounter`
- `SimpleGauge`
- `SimpleHistogram`

Also vector extensions of the above metric types are available and wrapped
as :

- `MultiCounter`
- `MultiGauge`
- `MultiHistogram`

The vector extensions basically maintain multiple labelled metrics in a single 
entity.

The crate only supports global metrics registry (singleton) and cannot 
create other standalone registries.

### JS/TS bindings

Because the crate is WASM compatible, it contains also TS/JS bindings for
`wasm-bindgen`.
Once a metric is created, it lives in a global registry. Values of all
created metrics can be gathered in a serialized text for at any time.

See the example below for details.

#### Example use in JS/TS

```js
const metric_counter = create_counter(
  'test_counter',
  'Some testing counter'
)

// Counter can be only incremented by integeres only
metric_counter.increment_by(10)

const metric_gauge = create_counter(
        'test_gauge',
        'Some testing gauge'
)

// Gauges can be incremented and decrements and support floats
metric_gauge.increment_by(5)
metric_gauge.decrement_by(3.2)

const metric_histogram = create_histogram(
        'test_histogram',
        'Some testing histogram'
)

// Histograms can observe floating point values
metric_histogram.observe(10.1)

// ... and also can be used to measure time durations in seconds
const timer = metric_gauge.start_measure()
foo()
metric_gauge.record_measure(timer)


// Multi-metrics are labeled extensions
const metric_countsPerVersion = create_multi_counter(
    'test_multi_counter',
    'Testing labeled counter',
    ['version']
)

// Tracks counters per different versions
metric_countsPerVersion.increment_by('1.0.0', 2)
metric_countsPerVersion.increment_by('1.0.1', 1)

// All metrics live in a global state and can be serialized at any time
let gathered_metrics = gather_all_metrics()
```