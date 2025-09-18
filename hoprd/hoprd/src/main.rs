use std::{collections::HashMap, fmt::Formatter, str::FromStr, sync::Arc};

use async_lock::RwLock;
use async_signal::{Signal, Signals};
use futures::{StreamExt, future::AbortHandle};
use hopr_lib::{HoprKeys, HoprLibProcesses, IdentityRetrievalModes, ToHex};
use hoprd::{cli::CliArgs, errors::HoprdError, exit::HoprServerIpForwardingReactor};
use hoprd_api::{ListenerJoinHandles, RestApiParameters, serve_api};
use signal_hook::low_level;
use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
#[cfg(feature = "telemetry")]
use {
    opentelemetry::trace::TracerProvider,
    opentelemetry_otlp::WithExportConfig as _,
    opentelemetry_sdk::trace::{RandomIdGenerator, Sampler},
};

// Avoid musl's default allocator due to degraded performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(any(target_env = "musl", target_env = "gnu"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
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

                let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(
                        opentelemetry_sdk::Resource::builder()
                            .with_service_name(
                                std::env::var("OTEL_SERVICE_NAME").unwrap_or(env!("CARGO_PKG_NAME").into()),
                            )
                            .build(),
                    )
                    .build()
                    .tracer(env!("CARGO_PKG_NAME"));

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

enum HoprdProcesses {
    HoprLib(HoprLibProcesses, AbortHandle),
    ListenerSockets(ListenerJoinHandles),
    RestApi(AbortHandle),
}

// Manual implementation needed, since Strum does not support skipping arguments
impl std::fmt::Display for HoprdProcesses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HoprdProcesses::HoprLib(p, _) => write!(f, "HoprLib process: {p}"),
            HoprdProcesses::ListenerSockets(_) => write!(f, "SessionListenerSockets"),
            HoprdProcesses::RestApi(_) => write!(f, "RestApi"),
        }
    }
}

impl std::fmt::Debug for HoprdProcesses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as Display
        write!(f, "{self}")
    }
}

#[cfg_attr(feature = "runtime-tokio", tokio::main)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    if hopr_crypto_random::is_rng_fixed() {
        warn!("!! FOR TESTING ONLY !! THIS BUILD IS USING AN INSECURE FIXED RNG !!")
    }

    if std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true") {
        warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");
    }

    if cfg!(debug_assertions) {
        warn!("Executable was built using the DEBUG profile.");
    } else {
        info!("Executable was built using the RELEASE profile.");
    }

    let args = <CliArgs as clap::Parser>::parse();
    let cfg = hoprd::config::HoprdConfig::from_cli_args(args, false)?;

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

    if let hopr_lib::HostType::IPv4(address) = &cfg.hopr.host.address {
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

    // Create the node instance
    info!("Creating the HOPRd node instance from hopr-lib");
    let node = Arc::new(hopr_lib::Hopr::new(
        cfg.clone().into(),
        &hopr_keys.packet_key,
        &hopr_keys.chain_key,
    )?);

    let node_clone = node.clone();

    let mut processes: Vec<HoprdProcesses> = Vec::new();

    if cfg.api.enable {
        let node_cfg_value =
            serde_json::to_value(cfg.as_redacted()).map_err(|e| HoprdError::ConfigError(e.to_string()))?;

        let api_cfg = cfg.api.clone();

        let listen_address = match &cfg.api.host.address {
            hopr_lib::HostType::IPv4(a) | hopr_lib::HostType::Domain(a) => {
                format!("{a}:{}", cfg.api.host.port)
            }
        };

        let api_listener = tokio::net::TcpListener::bind(&listen_address).await.map_err(|e| {
            hopr_lib::errors::HoprLibError::GeneralError(format!("REST API bind failed for {listen_address}: {e}"))
        })?;

        info!(listen_address, "Running a REST API");

        let session_listener_sockets = Arc::new(RwLock::new(HashMap::new()));

        processes.push(HoprdProcesses::ListenerSockets(session_listener_sockets.clone()));

        processes.push(HoprdProcesses::RestApi(hopr_async_runtime::spawn_as_abortable!(
            async move {
                if let Err(e) = serve_api(RestApiParameters {
                    listener: api_listener,
                    hoprd_cfg: node_cfg_value,
                    cfg: api_cfg,
                    hopr: node_clone,
                    session_listener_sockets,
                    default_session_listen_host: cfg.session_ip_forwarding.default_entry_listen_host,
                })
                .await
                {
                    error!(error = %e, "the REST API server could not start")
                }
            }
        )));
    }

    let (_hopr_socket, hopr_processes) = node
        .run(HoprServerIpForwardingReactor::new(
            hopr_keys.packet_key.clone(),
            cfg.session_ip_forwarding,
        ))
        .await?;

    processes.extend(hopr_processes.into_iter().map(|(k, v)| HoprdProcesses::HoprLib(k, v)));

    let mut signals = Signals::new([Signal::Hup, Signal::Int]).map_err(|e| HoprdError::OsError(e.to_string()))?;
    while let Some(Ok(signal)) = signals.next().await {
        match signal {
            Signal::Hup => {
                info!("Received the HUP signal... not doing anything");
            }
            Signal::Int => {
                info!("Received the INT signal... tearing down the node");
                futures::stream::iter(processes)
                    .then(|process| async move {
                        let mut abort_handles: Vec<AbortHandle> = Vec::new();
                        info!("Stopping process '{process}'");
                        match process {
                            HoprdProcesses::HoprLib(_, ah) | HoprdProcesses::RestApi(ah) => abort_handles.push(ah),
                            HoprdProcesses::ListenerSockets(ahs) => {
                                abort_handles
                                    .extend(ahs.write_arc().await.drain().map(|(_, entry)| entry.abort_handle));
                            }
                        }
                        futures::stream::iter(abort_handles)
                    })
                    .flatten()
                    .for_each_concurrent(None, |ah| async move { ah.abort() })
                    .await;

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
