# hopr-utils-tracing

Reusable tracing context envelopes and bounded queue adapters, independent of transport protocols
and message payloads. `ContextSender` captures producer context and `ContextSink` restores it for
downstream work; `WithContext` also supports explicitly supplied spans.

The default-enabled `tracing-context` feature controls automatic capture in `WithContext::from`.
With `default-features = false`, automatic capture stores a disabled span; explicit spans supplied
through `WithContext::new` remain available.

## License

GPL-3.0-only
