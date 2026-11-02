//! Capture structured records and formatted ancestry using the same subscriber.
//!
//! Tests asserting routing event counts use `serial_test::serial(routing_logs)` to isolate
//! subscriber registration and its process-wide callsite cache from other capture tests.

use std::{collections::BTreeMap, io::Write, sync::Arc};

use parking_lot::Mutex;
use test_log::tracing_subscriber::{self as subscriber, Layer, layer::SubscriberExt, registry::LookupSpan};
use tracing::{Subscriber, field::Visit};

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
}

pub(super) struct Record {
    pub fields: BTreeMap<String, String>,
    pub ancestry: Vec<&'static str>,
}

#[derive(Clone, Default)]
pub(super) struct Logs {
    events: Arc<Mutex<Vec<Record>>>,
    output: Arc<Mutex<Vec<u8>>>,
}

impl Logs {
    pub fn subscriber(&self, level: tracing::Level) -> impl Subscriber + Send + Sync + 'static {
        let output = self.output.clone();
        subscriber::registry()
            .with(self.clone())
            .with(
                subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time()
                    .with_writer(move || Writer(output.clone())),
            )
            // Scoped test subscribers run concurrently with different filters. Check the active
            // subscriber on each event rather than caching a callsite's interest at a fixed level.
            .with(subscriber::filter::dynamic_filter_fn(move |metadata, _| {
                *metadata.level() <= level
            }))
    }

    pub fn records(&self, message: &str) -> Vec<Record> {
        self.events
            .lock()
            .iter()
            .filter(|event| event.fields.get("message").is_some_and(|value| value == message))
            .map(|event| Record {
                fields: event.fields.clone(),
                ancestry: event.ancestry.clone(),
            })
            .collect()
    }

    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.output.lock()).into_owned()
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Logs {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: subscriber::layer::Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let ancestry = ctx
            .event_scope(event)
            .map_or_else(Vec::new, |scope| scope.from_root().map(|span| span.name()).collect());
        self.events.lock().push(Record {
            fields: fields.0,
            ancestry,
        });
    }
}

struct Writer(Arc<Mutex<Vec<u8>>>);

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Keep subscriber registration stress isolated from tests asserting routing event counts.
    #[test]
    #[serial_test::serial(routing_logs)]
    fn concurrent_scoped_subscribers_should_capture_their_own_levels() {
        fn emit() {
            tracing::debug!("debug capture check");
            tracing::trace!("trace capture check");
        }

        std::thread::scope(|scope| {
            for index in 0..8 {
                scope.spawn(move || {
                    let level = if index % 2 == 0 {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::TRACE
                    };
                    for _ in 0..20 {
                        let logs = Logs::default();
                        let _subscriber = tracing::subscriber::set_default(logs.subscriber(level));
                        emit();
                        assert_eq!(logs.records("debug capture check").len(), 1);
                        let traces = logs.records("trace capture check");
                        assert_eq!(traces.len(), usize::from(level == tracing::Level::TRACE));
                        assert!(logs.records("debug capture check")[0].ancestry.is_empty());
                    }
                });
            }
        });
    }

    #[test]
    fn buffered_log_output_should_survive_flush_and_append_from_another_writer() -> std::io::Result<()> {
        let logs = Logs::default();
        let mut writer = std::io::BufWriter::new(Writer(logs.output.clone()));
        write!(writer, "candidate set ")?;
        writeln!(writer, "installed set_id={}", 42)?;
        assert!(logs.output().is_empty(), "buffered fragments must wait for flush");
        writer.flush()?;
        assert_eq!(logs.output(), "candidate set installed set_id=42\n");

        let mut other = Writer(logs.output.clone());
        writeln!(other, "path selected set_id=42")?;
        other.flush()?;
        assert_eq!(
            logs.output(),
            "candidate set installed set_id=42\npath selected set_id=42\n"
        );
        Ok(())
    }
}
