// Disabled until `wasm-bindgen-test-runner` supports ESM

// use js_sys::Atomics::wait_with_timeout;
// use js_sys::{Int32Array, SharedArrayBuffer};
// use wasm_bindgen_test::*;

// use utils_metrics::metrics::wasm::*;

// fn sleep(millis: f64) {
//     // Workaround for missing timeout in WASM
//     let sab = SharedArrayBuffer::new(1024);
//     let arr = Int32Array::new(&sab);
//     wait_with_timeout(&arr, 0, 0, millis).unwrap();
// }

// #[wasm_bindgen_test]
// fn test_histogram_timer() {
//     let histogram = create_histogram_with_buckets(
//         "my_histogram",
//         "test description",
//         vec![1.0, 2.0, 3.0, 4.0].as_slice(),
//     )
//     .unwrap();

//     let timer = histogram.start_measure();
//     sleep(2500.0);
//     histogram.record_measure(timer);

//     let metrics = gather_all_metrics().unwrap();
//     assert!(metrics.contains("my_histogram_bucket{le=\"1\"} 0"));
//     assert!(metrics.contains("my_histogram_bucket{le=\"2\"} 0"));
//     assert!(metrics.contains("my_histogram_bucket{le=\"3\"} 1"));
//     assert!(metrics.contains("my_histogram_bucket{le=\"4\"} 1"));
// }
