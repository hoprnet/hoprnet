use std::sync::OnceLock;

use tracing_subscriber::{Registry, prelude::*, reload};

/// OTEL tracing layer that operates on top of the base [`Registry`] subscriber.
type OtelBoxedLayer = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static>;

/// Stored once so that [`install_otel_layers`] can upgrade the subscriber after the Tokio
/// runtime is running (and OTEL batch processors have been created).
static OTEL_HANDLE: OnceLock<reload::Handle<Vec<OtelBoxedLayer>, Registry>> = OnceLock::new();

fn passthrough_layers(layers: Vec<OtelBoxedLayer>) -> Vec<OtelBoxedLayer> {
    if layers.is_empty() {
        vec![Box::new(tracing_subscriber::layer::Identity::new())]
    } else {
        layers
    }
}

/// Install the base tracing subscriber with a reload slot for OTEL layers.
pub(super) fn install_base_subscriber() -> anyhow::Result<()> {
    let env_filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => tracing_subscriber::filter::EnvFilter::new("info")
            .add_directive("libp2p_swarm=info".parse()?)
            .add_directive("libp2p_mplex=info".parse()?)
            .add_directive("libp2p_tcp=info".parse()?)
            .add_directive("libp2p_dns=info".parse()?)
            .add_directive("multistream_select=info".parse()?)
            .add_directive("isahc=error".parse()?)
            .add_directive("sea_orm=warn".parse()?)
            .add_directive("sqlx=warn".parse()?)
            .add_directive("hyper_util=warn".parse()?),
    };

    #[cfg(feature = "prof")]
    let env_filter = env_filter
        .add_directive("tokio=trace".parse()?)
        .add_directive("runtime=trace".parse()?);

    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let format = if std::env::var("HOPRD_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false)
    {
        format.json().boxed()
    } else {
        format.boxed()
    };

    #[cfg(feature = "prof")]
    let prof_layer = console_subscriber::spawn();
    #[cfg(not(feature = "prof"))]
    let prof_layer = tracing_subscriber::layer::Identity::new();

    let (reload_layer, handle) = reload::Layer::<Vec<OtelBoxedLayer>, Registry>::new(passthrough_layers(Vec::new()));
    let _ = OTEL_HANDLE.set(handle); // ignore the error if called more than once (e.g. in tests)

    let subscriber = Registry::default()
        .with(reload_layer)
        .with(env_filter)
        .with(prof_layer)
        .with(format);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Slot OTEL layers into the already-running base subscriber via the reload handle.
/// Must be called after [`install_base_subscriber`] and after the Tokio runtime is up
/// (OTEL batch processors require it).
pub(super) fn install_otel_layers(layers: Vec<OtelBoxedLayer>) -> anyhow::Result<()> {
    OTEL_HANDLE
        .get()
        .ok_or_else(|| anyhow::anyhow!("base subscriber not initialized; call install_base_subscriber() first"))?
        .reload(passthrough_layers(layers))?;
    Ok(())
}
