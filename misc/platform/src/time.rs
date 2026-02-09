pub mod native {
    pub fn current_time() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
}

pub use native::current_time;

/// Wraps an expression with optional TRACE-level timing instrumentation.
///
/// When the `trace-timing` feature is enabled, the macro measures the elapsed
/// time of the body expression and emits a `tracing::trace!` log with an
/// `elapsed_ms` field. The timing overhead is further gated behind
/// `tracing::enabled!(tracing::Level::TRACE)` so that `Instant::now()` is
/// only called when a TRACE subscriber is active.
///
/// When the feature is disabled, the macro evaluates to just the body.
///
/// # Variants
///
/// ```ignore
/// // Simple: label + body
/// let val = trace_timed!("my_operation", { some_async_fn().await? });
///
/// // With extra tracing fields
/// let val = trace_timed!("my_operation", packet_type = "fwd", { body });
/// ```
#[cfg(feature = "trace-timing")]
#[macro_export]
macro_rules! trace_timed {
    ($label:expr, { $($body:tt)* }) => {{
        let __trace_timing = tracing::enabled!(tracing::Level::TRACE);
        let __trace_start = __trace_timing.then(std::time::Instant::now);
        let __trace_result = { $($body)* };
        if let Some(__start) = __trace_start {
            tracing::trace!(elapsed_ms = __start.elapsed().as_millis() as u64, $label);
        }
        __trace_result
    }};
    ($label:expr, $($field:ident = $val:expr),+, { $($body:tt)* }) => {{
        let __trace_timing = tracing::enabled!(tracing::Level::TRACE);
        let __trace_start = __trace_timing.then(std::time::Instant::now);
        let __trace_result = { $($body)* };
        if let Some(__start) = __trace_start {
            tracing::trace!(elapsed_ms = __start.elapsed().as_millis() as u64, $($field = $val),+, $label);
        }
        __trace_result
    }};
}

#[cfg(not(feature = "trace-timing"))]
#[macro_export]
macro_rules! trace_timed {
    ($label:expr, { $($body:tt)* }) => {{ $($body)* }};
    ($label:expr, $($field:ident = $val:expr),+, { $($body:tt)* }) => {{ $($body)* }};
}
