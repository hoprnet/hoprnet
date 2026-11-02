//! Capture structured records and formatted ancestry using the same subscriber.

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
            .with(subscriber::filter::LevelFilter::from_level(level))
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
