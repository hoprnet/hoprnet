use async_lock::RwLock;
use async_signal::{Signal, Signals};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::str::FromStr;
use std::{sync::Arc, time::SystemTime};
#[cfg(feature = "telemetry")]
use {
    opentelemetry::trace::TracerProvider,
    opentelemetry_otlp::WithExportConfig as _,
    opentelemetry_sdk::trace::{RandomIdGenerator, Sampler},
};

use signal_hook::low_level;
use tracing::{error, info, trace, warn};
use tracing_subscriber::prelude::*;

use hopr_async_runtime::prelude::{cancel_join_handle, spawn, JoinHandle};
use hopr_lib::{ApplicationData, AsUnixTimestamp, HoprLibProcesses, ToHex};
use hopr_platform::file::native::join;
use hoprd::cli::CliArgs;
use hoprd::errors::HoprdError;
use hoprd_api::{serve_api, ListenerJoinHandles, RestApiParameters};
use hoprd_db_api::aliases::{HoprdDbAliasesOperations, ME_AS_ALIAS};
use hoprd_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};

use hoprd::exit::HoprServerIpForwardingReactor;

const WEBSOCKET_EVENT_BROADCAST_CAPACITY: usize = 10000;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleHistogram;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_MESSAGE_LATENCY: SimpleHistogram = SimpleHistogram::new(
        "hopr_message_latency_sec",
        "Histogram of measured received message latencies in seconds",
        vec![0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0]
    ).unwrap();
}

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
            .add_directive("surf::middleware::logger::native=error".parse()?)
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
                let tracer = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(
                        opentelemetry_otlp::new_exporter()
                            .tonic()
                            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
                            .with_timeout(std::time::Duration::from_secs(5)),
                    )
                    .with_trace_config(
                        opentelemetry_sdk::trace::Config::default()
                            .with_sampler(Sampler::AlwaysOn)
                            .with_id_generator(RandomIdGenerator::default())
                            .with_max_events_per_span(64)
                            .with_max_attributes_per_span(16)
                            .with_resource(opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                                "service.name",
                                std::env::var("OTEL_SERVICE_NAME").unwrap_or(env!("CARGO_PKG_NAME").into()),
                            )])),
                    )
                    .install_batch(opentelemetry_sdk::runtime::Tokio)?
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
    HoprLib(HoprLibProcesses, JoinHandle<()>),
    Inbox(JoinHandle<()>),
    ListenerSockets(ListenerJoinHandles),
    RestApi(JoinHandle<()>),
}

// Manual implementation needed, since Strum does not support skipping arguments
impl std::fmt::Display for HoprdProcesses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HoprdProcesses::HoprLib(p, _) => write!(f, "HoprLib process: {p}"),
            HoprdProcesses::Inbox(_) => write!(f, "Inbox"),
            HoprdProcesses::ListenerSockets(_) => write!(f, "SessionListenerSockets"),
            HoprdProcesses::RestApi(_) => write!(f, "RestApi"),
        }
    }
}

impl std::fmt::Debug for HoprdProcesses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Intentionally same as Display
        write!(f, "{}", self)
    }
}

#[cfg_attr(all(feature = "runtime-tokio", not(feature = "runtime-async-std")), tokio::main)]
#[cfg_attr(feature = "runtime-async-std", async_std::main)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    if hopr_crypto_random::is_rng_fixed() {
        warn!("!! FOR TESTING ONLY !! THIS BUILD IS USING AN INSECURE FIXED RNG !!")
    }

    if std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true") {
        warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");
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

    // TODO: the following check can be removed once [PR](https://github.com/hoprnet/hoprnet/pull/5665) is merged
    if hopr_lib::Keypair::public(&hopr_keys.packet_key)
        .to_string()
        .starts_with("0xff")
    {
        warn!("This node uses an invalid packet key type and will not be able to become an effective relay node, please create a new identity!");
    }

    // Create the node instance
    info!("Creating the HOPRd node instance from hopr-lib");
    let node = Arc::new(hopr_lib::Hopr::new(
        cfg.clone().into(),
        &hopr_keys.packet_key,
        &hopr_keys.chain_key,
    )?);

    // Create the message inbox
    let inbox: Arc<RwLock<hoprd_inbox::Inbox>> = Arc::new(RwLock::new(
        hoprd_inbox::inbox::MessageInbox::new_with_time(cfg.inbox.clone(), || {
            hopr_platform::time::native::current_time().as_unix_timestamp()
        }),
    ));

    // Create the metadata database
    let db_path: String = join(&[&cfg.hopr.db.data, "db"]).expect("Could not create a db storage path");

    let hoprd_db = match hoprd_db_api::db::HoprdDb::new(db_path.clone()).await {
        Ok(db) => {
            info!("Metadata database created successfully");
            Arc::new(db)
        }
        Err(e) => {
            error!(error = %e, "Failed to create the metadata database");
            return Err(e.into());
        }
    };

    // Ensures that "OWN_ALIAS" is set as alias
    match hoprd_db
        .set_alias(node.me_peer_id().to_string(), ME_AS_ALIAS.to_string())
        .await
    {
        Ok(_) => {
            info!("Own alias set successfully");
        }
        Err(hoprd_db_api::errors::DbError::ReAliasingSelfNotAllowed) => {
            info!("Own alias already set");
        }
        Err(e) => {
            error!(error = %e, "Failed to set the alias for the node");
        }
    }

    let (mut ws_events_tx, ws_events_rx) =
        async_broadcast::broadcast::<ApplicationData>(WEBSOCKET_EVENT_BROADCAST_CAPACITY);
    let ws_events_rx = ws_events_rx.deactivate(); // No need to copy the data unless the websocket is opened, but leaves the channel open
    ws_events_tx.set_overflow(true); // Set overflow in case of full the oldest record is discarded

    let inbox_clone = inbox.clone();

    let node_clone = node.clone();

    let mut processes: Vec<HoprdProcesses> = Vec::new();

    if cfg.api.enable {
        // TODO: remove RLP in 3.0
        let msg_encoder =
            |data: &[u8]| hopr_lib::rlp::encode(data, hopr_platform::time::native::current_time().as_unix_timestamp());

        let node_cfg_str = cfg.as_redacted_string()?;
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
        processes.push(HoprdProcesses::RestApi(spawn(async move {
            if let Err(e) = serve_api(RestApiParameters {
                listener: api_listener,
                hoprd_cfg: node_cfg_str,
                cfg: api_cfg,
                hopr: node_clone,
                hoprd_db,
                inbox,
                session_listener_sockets,
                websocket_rx: ws_events_rx,
                msg_encoder: Some(msg_encoder),
                default_session_listen_host: cfg.session_ip_forwarding.default_entry_listen_host,
            })
            .await
            {
                error!(error = %e, "the REST API server could not start")
            }
        })));
    }

    let (hopr_socket, hopr_processes) = node
        .run(HoprServerIpForwardingReactor::new(
            hopr_keys.packet_key.clone(),
            cfg.session_ip_forwarding,
        ))
        .await?;

    // process extracting the received data from the socket
    let mut ingress = hopr_socket.reader();
    processes.push(HoprdProcesses::Inbox(spawn(async move {
        while let Some(data) = ingress.next().await {
            let recv_at = SystemTime::now();

            // TODO: remove RLP in 3.0
            match hopr_lib::rlp::decode(&data.plain_text) {
                Ok((msg, sent)) => {
                    let latency = recv_at.as_unix_timestamp().saturating_sub(sent);

                    trace!(
                        tag = ?data.application_tag,
                        latency_in_ms = latency.as_millis(),
                        receiged_at = DateTime::<Utc>::from(recv_at).to_rfc3339(),
                        "received message"
                    );

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_MESSAGE_LATENCY.observe(latency.as_secs_f64());

                    if cfg.api.enable && ws_events_tx.receiver_count() > 0 {
                        if let Err(e) = ws_events_tx.try_broadcast(ApplicationData {
                            application_tag: data.application_tag,
                            plain_text: msg.clone(),
                        }) {
                            error!(error = %e, "Failed to notify websockets about a new message");
                        }
                    }

                    if !inbox_clone
                        .write()
                        .await
                        .push(ApplicationData {
                            application_tag: data.application_tag,
                            plain_text: msg,
                        })
                        .await
                    {
                        warn!(
                            tag = data.application_tag,
                            "Received a message with an ignored Inbox tag",
                        )
                    }
                }
                Err(e) => error!(error = %e, "RLP decoding failed"),
            }
        }
    })));

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
                        let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
                        info!("Stopping process '{process}'");
                        match process {
                            HoprdProcesses::HoprLib(_, jh)
                            | HoprdProcesses::Inbox(jh)
                            | HoprdProcesses::RestApi(jh) => join_handles.push(jh),
                            HoprdProcesses::ListenerSockets(jhs) => {
                                join_handles.extend(jhs.write().await.drain().map(|(_, entry)| entry.jh));
                            }
                        }
                        futures::stream::iter(join_handles)
                    })
                    .flatten()
                    .for_each_concurrent(None, cancel_join_handle)
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
