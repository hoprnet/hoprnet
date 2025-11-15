use std::{path::PathBuf, str::FromStr, sync::Arc};

use async_signal::{Signal, Signals};
use futures::{FutureExt, StreamExt, channel::mpsc::channel, future::abortable};
use hopr_chain_connector::{
    BlockchainConnectorConfig, HoprBlockchainSafeConnector,
    blokli_client::{BlokliClient, BlokliClientConfig},
};
use hopr_db_node::{HoprNodeDb, HoprNodeDbConfig};
use hopr_lib::{
    AbortableList, AcknowledgedTicket, HoprKeys, IdentityRetrievalModes, Keypair, ToHex, errors::HoprLibError,
    exports::api::chain::ChainEvents, prelude::ChainKeypair, state::HoprLibProcess,
    utils::session::ListenerJoinHandles,
};
use hoprd::{
    cli::CliArgs,
    config::{DEFAULT_BLOKLI_URL, HoprdConfig},
    errors::HoprdError,
    exit::HoprServerIpForwardingReactor,
};
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
    #[strum(to_string = "hopr-lib process: {0}")]
    HoprLib(HoprLibProcess),
    #[strum(to_string = "session listener sockets")]
    ListenerSocket,
    #[strum(to_string = "hopr strategies process")]
    Strategies,
    #[strum(to_string = "REST API process")]
    RestApi,
}

#[cfg(not(feature = "runtime-tokio"))]
compile_error!("The 'runtime-tokio' feature must be enabled");

async fn init_db(
    cfg: &HoprdConfig,
    chain_key: &ChainKeypair,
) -> Result<(HoprNodeDb, futures::channel::mpsc::Receiver<AcknowledgedTicket>), anyhow::Error> {
    let db_path: PathBuf = [&cfg.db.data, "node_db"].iter().collect();
    info!(path = ?db_path, "initiating DB");

    let mut create_if_missing = cfg.db.initialize;

    if cfg.db.force_initialize {
        info!("Force cleaning up existing database");
        std::fs::remove_dir_all(db_path.as_path())?;
        create_if_missing = true;
    }

    // create DB dir if it does not exist
    if let Some(parent_dir_path) = db_path.as_path().parent() {
        if !parent_dir_path.is_dir() {
            std::fs::create_dir_all(parent_dir_path).map_err(|e| {
                HoprLibError::GeneralError(format!(
                    "Failed to create DB parent directory at '{parent_dir_path:?}': {e}"
                ))
            })?
        }
    }

    let db_cfg = HoprNodeDbConfig {
        create_if_missing,
        force_create: cfg.db.force_initialize,
        log_slow_queries: std::time::Duration::from_millis(150),
        surb_ring_buffer_size: std::env::var("HOPR_PROTOCOL_SURB_RB_SIZE")
            .ok()
            .and_then(|s| u64::from_str(&s).map(|v| v as usize).ok())
            .unwrap_or_else(|| HoprNodeDbConfig::default().surb_ring_buffer_size),
        surb_distress_threshold: std::env::var("HOPR_PROTOCOL_SURB_RB_DISTRESS")
            .ok()
            .and_then(|s| u64::from_str(&s).map(|v| v as usize).ok())
            .unwrap_or_else(|| HoprNodeDbConfig::default().surb_distress_threshold),
    };
    let node_db = HoprNodeDb::new(db_path.as_path(), chain_key.clone(), db_cfg).await?;

    let ack_ticket_channel_capacity = std::env::var("HOPR_INTERNAL_ACKED_TICKET_CHANNEL_CAPACITY")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&c| c > 0)
        .unwrap_or(2048);

    debug!(
        capacity = ack_ticket_channel_capacity,
        "starting winning ticket processing"
    );
    let (on_ack_tkt_tx, on_ack_tkt_rx) = channel::<AcknowledgedTicket>(ack_ticket_channel_capacity);
    node_db.start_ticket_processing(Some(on_ack_tkt_tx))?;

    Ok((node_db, on_ack_tkt_rx))
}

async fn init_blokli_connector(
    chain_key: &ChainKeypair,
    cfg: &HoprdConfig,
) -> anyhow::Result<HoprBlockchainSafeConnector<BlokliClient>> {
    // TODO: instantiate the connector properly
    info!("initiating Blokli connector");
    let mut connector = hopr_chain_connector::create_trustful_hopr_blokli_connector(
        chain_key,
        BlockchainConnectorConfig {
            tx_confirm_timeout: std::time::Duration::from_secs(30),
            ..Default::default()
        },
        BlokliClient::new(
            cfg.provider
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(DEFAULT_BLOKLI_URL)
                .parse()?,
            BlokliClientConfig {
                timeout: std::time::Duration::from_secs(5),
            },
        ),
        cfg.hopr.safe_module.module_address,
    )
    .await?;
    connector.connect(std::time::Duration::from_secs(30)).await?;

    Ok(connector)
}

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

    let session_listener_sockets = Arc::new(ListenerJoinHandles::default());

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
    hopr_lib::prepare_tokio_runtime()?.block_on(main_inner())
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

    if let hopr_lib::config::HostType::IPv4(address) = &cfg.hopr.host.address {
        let ipv4 = std::net::Ipv4Addr::from_str(address).map_err(|e| HoprdError::ConfigError(e.to_string()))?;

        if ipv4.is_loopback() && !cfg.hopr.transport.announce_local_addresses {
            Err(hopr_lib::errors::HoprLibError::GeneralError(
                "Cannot announce a loopback address".into(),
            ))?;
        }
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
    let (node_db, stored_tickets) = init_db(&cfg, &hopr_keys.chain_key).await?;

    let chain_connector = Arc::new(init_blokli_connector(&hopr_keys.chain_key, &cfg).await?);

    // Create the node instance
    info!("creating the HOPRd node instance from hopr-lib");
    let node = Arc::new(
        hopr_lib::Hopr::new(
            cfg.clone().into(),
            chain_connector.clone(),
            node_db.clone(),
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

    let (_hopr_socket, hopr_lib_processes) = node
        .run(HoprServerIpForwardingReactor::new(
            hopr_keys.packet_key.clone(),
            cfg.session_ip_forwarding,
        ))
        .await?;

    let multi_strategy = Arc::new(hopr_strategy::strategy::MultiStrategy::new(
        cfg.strategy.clone(),
        chain_connector.clone(),
        node.redemption_requests()?,
    ));
    debug!(strategies = ?multi_strategy, "initialized strategies");

    processes.flat_map_extend_from(hopr_lib_processes, HoprdProcess::HoprLib);

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
                processes.abort_all();

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
