use tracing_subscriber::prelude::*;

pub(super) fn build_base_subscriber()
-> anyhow::Result<impl tracing::Subscriber + Send + Sync + for<'span> tracing_subscriber::registry::LookupSpan<'span>> {
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

    Ok(tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(prof_layer)
        .with(format))
}
