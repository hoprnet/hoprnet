use std::{num::NonZeroUsize, str::FromStr, sync::Arc};

use async_signal::{Signal, Signals};
use futures::{FutureExt, StreamExt, future::abortable};
use hopr_db_node::HoprNodeDb;
use hopr_lib::{
    AbortableList, HoprKeys, IdentityRetrievalModes, Keypair, ToHex,
    exports::api::chain::ChainEvents,
    utils::blokli::{BlokliClient, HoprBlockchainSafeConnector},
};
use hoprd::{cli::CliArgs, config::HoprdConfig, errors::HoprdError, exit::HoprServerIpForwardingReactor};
use hoprd_api::{RestApiParameters, serve_api};
use signal_hook::low_level;
use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;
#[cfg(feature = "telemetry")]
use {
    opentelemetry::trace::TracerProvider,
    opentelemetry_otlp::WithExportConfig as _,
    opentelemetry_sdk::trace::{RandomIdGenerator, Sampler},
};

// Avoid musl's default allocator due to degraded performance
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(target_os = "linux", feature = "allocator-jemalloc-stats"))]
mod jemalloc_stats;

fn init_logger() -> anyhow::Result<()> {
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
    let registry = tracing_subscriber::Registry::default()
        .with(
            env_filter
                .add_directive("tokio=trace".parse()?)
                .add_directive("runtime=trace".parse()?),
        )
        .with(console_subscriber::spawn());

    #[cfg(not(feature = "prof"))]
    let registry = tracing_subscriber::Registry::default().with(env_filter);

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

    let registry = registry.with(format);

    let mut telemetry = None;

    #[cfg(feature = "telemetry")]
    {
        match std::env::var("HOPRD_USE_OPENTELEMETRY") {
            Ok(v) if v == "true" => {
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_protocol(opentelemetry_otlp::Protocol::Grpc)
                    .with_timeout(std::time::Duration::from_secs(5))
                    .build()?;
                let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or(env!("CARGO_PKG_NAME").into());

                let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(
                        opentelemetry_sdk::Resource::builder()
                            .with_service_name(service_name.to_string())
                            .build(),
                    )
                    .build()
                    .tracer(env!("CARGO_PKG_NAME"));
                info!(
                    otel_service_name = %service_name,
                    otel_exporter = "otlp",
                    "OpenTelemetry tracing enabled"
                );
                telemetry = Some(tracing_opentelemetry::layer().with_tracer(tracer))
            }
            _ => {}
        }
    }

    if let Some(telemetry) = telemetry {
        tracing::subscriber::set_global_default(registry.with(telemetry))?
    } else {
        tracing::subscriber::set_global_default(registry)?
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
enum HoprdProcess {
    #[strum(to_string = "session listener sockets")]
    ListenerSocket,
    #[strum(to_string = "hopr strategies process")]
    Strategies,
    #[strum(to_string = "REST API process")]
    RestApi,
}

#[cfg(not(feature = "runtime-tokio"))]
compile_error!("The 'runtime-tokio' feature must be enabled");

async fn init_rest_api(
    cfg: &HoprdConfig,
    hopr: Arc<hopr_lib::Hopr<Arc<HoprBlockchainSafeConnector<BlokliClient>>, HoprNodeDb>>,
) -> anyhow::Result<AbortableList<HoprdProcess>> {
    let node_cfg_value = serde_json::to_value(cfg.as_redacted()).map_err(|e| HoprdError::ConfigError(e.to_string()))?;

    let api_cfg = cfg.api.clone();

    let listen_address = match &cfg.api.host.address {
        hopr_lib::config::HostType::IPv4(a) | hopr_lib::config::HostType::Domain(a) => {
            format!("{a}:{}", cfg.api.host.port)
        }
    };

    let api_listener = tokio::net::TcpListener::bind(&listen_address).await.map_err(|e| {
        hopr_lib::errors::HoprLibError::GeneralError(format!("REST API bind failed for {listen_address}: {e}"))
    })?;

    info!(listen_address, "Running a REST API");

    let session_listener_sockets = Arc::new(hopr_utils_session::ListenerJoinHandles::default());

    let mut processes = AbortableList::<HoprdProcess>::default();

    processes.insert(HoprdProcess::ListenerSocket, session_listener_sockets.clone());

    let cfg_clone = cfg.clone();
    let (proc, abort_handle) = abortable(
        async move {
            if let Err(e) = serve_api(RestApiParameters {
                listener: api_listener,
                hoprd_cfg: node_cfg_value,
                cfg: api_cfg,
                hopr,
                session_listener_sockets,
                default_session_listen_host: cfg_clone.session_ip_forwarding.default_entry_listen_host,
            })
            .await
            {
                error!(error = %e, "the REST API server could not start")
            }
        }
        .inspect(|_| tracing::warn!(task = "hoprd - REST API", "long-running background task finished")),
    );
    let _jh = tokio::spawn(proc);
    processes.insert(HoprdProcess::RestApi, abort_handle);

    Ok(processes)
}

#[cfg(feature = "runtime-tokio")]
fn main() -> anyhow::Result<()> {
    let num_cpu_threads = std::env::var("HOPRD_NUM_CPU_THREADS").ok().and_then(|v| {
        usize::from_str(&v)
            .map_err(anyhow::Error::from)
            .and_then(|v| NonZeroUsize::try_from(v).map_err(anyhow::Error::from))
            .inspect_err(|error| error!(%error, "failed to parse HOPRD_NUM_CPU_THREADS"))
            .ok()
    });

    let num_io_threads = std::env::var("HOPRD_NUM_IO_THREADS").ok().and_then(|v| {
        usize::from_str(&v)
            .map_err(anyhow::Error::from)
            .and_then(|v| NonZeroUsize::try_from(v).map_err(anyhow::Error::from))
            .inspect_err(|error| error!(%error, "failed to parse HOPRD_NUM_IO_THREADS"))
            .ok()
    });

    hopr_lib::prepare_tokio_runtime(num_cpu_threads, num_io_threads)?.block_on(main_inner())
}

#[cfg(feature = "runtime-tokio")]
async fn main_inner() -> anyhow::Result<()> {
    init_logger()?;

    #[cfg(all(target_os = "linux", feature = "allocator-jemalloc-stats"))]
    let _jemalloc_stats = jemalloc_stats::JemallocStats::start().await;

    if cfg!(debug_assertions) {
        warn!("Executable was built using the DEBUG profile.");
    } else {
        info!("Executable was built using the RELEASE profile.");
    }

    let args = <CliArgs as clap::Parser>::parse();
    let cfg = HoprdConfig::from_cli_args(args, false)?;

    let git_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
    info!(
        version = hopr_lib::constants::APP_VERSION,
        hash = git_hash,
        cfg = cfg.as_redacted_string()?,
        "Starting HOPR daemon"
    );

    if std::env::var("DAPPNODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        info!("The HOPRd node appears to run on DappNode");
    }

    // Find or create an identity
    let hopr_keys: HoprKeys = match &cfg.identity.private_key {
        Some(private_key) => IdentityRetrievalModes::FromPrivateKey { private_key },
        None => IdentityRetrievalModes::FromFile {
            password: &cfg.identity.password,
            id_path: &cfg.identity.file,
        },
    }
    .try_into()?;

    info!(
        packet_key = hopr_lib::Keypair::public(&hopr_keys.packet_key).to_peerid_str(),
        blockchain_address = hopr_lib::Keypair::public(&hopr_keys.chain_key).to_address().to_hex(),
        "Node public identifiers"
    );

    // TODO: stored tickets need to be emitted from the Hopr object (addressed in #7575)
    let (node_db, stored_tickets) = hopr_utils_node_db::init_db(
        &hopr_keys.chain_key,
        &cfg.db.data,
        cfg.db.initialize,
        cfg.db.force_initialize,
    )
    .await?;

    let chain_connector = Arc::new(
        hopr_lib::utils::blokli::init_blokli_connector(
            &hopr_keys.chain_key,
            cfg.provider.clone(),
            cfg.hopr.safe_module.module_address,
        )
        .await?,
    );

    // Create the node instance
    info!("creating the HOPRd node instance from hopr-lib");
    let node = Arc::new(
        hopr_lib::Hopr::new(
            cfg.clone().into(),
            chain_connector.clone(),
            node_db,
            &hopr_keys.packet_key,
            &hopr_keys.chain_key,
        )
        .await?,
    );

    let mut processes = AbortableList::<HoprdProcess>::default();

    if cfg.api.enable {
        let list = init_rest_api(&cfg, node.clone()).await?;
        processes.extend_from(list);
    }

    let _hopr_socket = node
        .run(
            None::<hopr_lib::DummyCoverTrafficType>,
            HoprServerIpForwardingReactor::new(hopr_keys.packet_key.clone(), cfg.session_ip_forwarding),
        )
        .await?;

    let multi_strategy = Arc::new(hopr_strategy::strategy::MultiStrategy::new(
        cfg.strategy.clone(),
        chain_connector.clone(),
        node.redemption_requests()?,
    ));
    debug!(strategies = ?multi_strategy, "initialized strategies");

    debug!("starting up strategies");
    processes.insert(
        HoprdProcess::Strategies,
        hopr_strategy::stream_events_to_strategy_with_tick(
            multi_strategy,
            chain_connector.subscribe()?,
            stored_tickets,
            cfg.strategy.execution_interval,
            hopr_keys.chain_key.public().to_address(),
        ),
    );

    let mut signals = Signals::new([Signal::Hup, Signal::Int]).map_err(|e| HoprdError::OsError(e.to_string()))?;
    while let Some(Ok(signal)) = signals.next().await {
        match signal {
            Signal::Hup => {
                info!("Received the HUP signal... not doing anything");
            }
            Signal::Int => {
                info!("Received the INT signal... tearing down the node");
                // Explicitly tear down running processes here
                drop(node);
                drop(processes);

                info!("All processes stopped... emulating the default handler...");
                low_level::emulate_default_handler(signal as i32)?;
                info!("Shutting down!");
                break;
            }
            _ => low_level::emulate_default_handler(signal as i32)?,
        }
    }

    Ok(())
}
