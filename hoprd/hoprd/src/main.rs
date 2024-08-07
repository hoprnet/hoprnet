use std::collections::HashMap;
use std::str::FromStr;
use std::{sync::Arc, time::SystemTime};

use async_lock::RwLock;
use async_signal::{Signal, Signals};
use chrono::{DateTime, Utc};
use futures::StreamExt;

#[cfg(feature = "telemetry")]
use {
    opentelemetry::trace::TracerProvider,
    opentelemetry_otlp::WithExportConfig as _,
    opentelemetry_sdk::trace::{RandomIdGenerator, Sampler},
};

use signal_hook::low_level;
use tracing::{error, info, warn};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use hopr_async_runtime::prelude::{cancel_join_handle, spawn, JoinHandle};
use hopr_lib::{ApplicationData, AsUnixTimestamp, HoprLibProcesses, ToHex, TransportOutput};
use hoprd::cli::CliArgs;
use hoprd::errors::HoprdError;
use hoprd_api::serve_api;
use hoprd_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};

use hoprd::HoprServerReactor;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleHistogram;

const WEBSOCKET_EVENT_BROADCAST_CAPACITY: usize = 10000;

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
            .add_directive("libp2p_mplex=info".parse()?)
            .add_directive("libp2p_swarm=info".parse()?)
            .add_directive("multistream_select=info".parse()?)
            .add_directive("isahc::handler=error".parse()?)
            .add_directive("isahc::client=error".parse()?)
            .add_directive("surf::middleware::logger::native=error".parse()?),
    };

    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let registry = tracing_subscriber::Registry::default().with(env_filter).with(format);

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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum HoprdProcesses {
    HoprLib(HoprLibProcesses),
    Socket,
    RestApi,
}

#[cfg_attr(feature = "runtime-async-std", async_std::main)]
#[cfg_attr(all(feature = "runtime-tokio", not(feature = "runtime-async-std")), tokio::main)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger()?;

    let args = <CliArgs as clap::Parser>::parse();
    let cfg = hoprd::config::HoprdConfig::from_cli_args(args, false)?;

    let git_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
    info!("This is HOPRd {} ({})", hopr_lib::constants::APP_VERSION, git_hash);

    if std::env::var("DAPPNODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        info!("The HOPRd node appears to run on DappNode");
    }

    info!("Node configuration: {}", cfg.as_redacted_string()?);

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
        "This node has packet key '{}' and uses a blockchain address '{}'",
        hopr_lib::Keypair::public(&hopr_keys.packet_key).to_peerid_str(),
        hopr_lib::Keypair::public(&hopr_keys.chain_key).to_address().to_hex()
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
    let hoprlib_cfg: hopr_lib::config::HoprLibConfig = cfg.clone().into();

    let node = Arc::new(hopr_lib::Hopr::new(
        hoprlib_cfg.clone(),
        &hopr_keys.packet_key,
        &hopr_keys.chain_key,
    ));

    // Create the message inbox
    let inbox: Arc<RwLock<hoprd_inbox::Inbox>> = Arc::new(RwLock::new(
        hoprd_inbox::inbox::MessageInbox::new_with_time(cfg.inbox.clone(), || {
            hopr_platform::time::native::current_time().as_unix_timestamp()
        }),
    ));

    let (mut ws_events_tx, ws_events_rx) =
        async_broadcast::broadcast::<TransportOutput>(WEBSOCKET_EVENT_BROADCAST_CAPACITY);
    let ws_events_rx = ws_events_rx.deactivate(); // No need to copy the data unless the websocket is opened, but leaves the channel open
    ws_events_tx.set_overflow(true); // Set overflow in case of full the oldest record is discarded

    let inbox_clone = inbox.clone();

    let node_clone = node.clone();

    let mut processes: HashMap<HoprdProcesses, JoinHandle<()>> = HashMap::new();

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

        let api_listener = tokio::net::TcpListener::bind(&listen_address)
            .await
            .unwrap_or_else(|e| panic!("REST API bind failed for {listen_address}: {e}"));

        info!("Node REST API is listening on {listen_address}");

        processes.insert(
            HoprdProcesses::RestApi,
            spawn(async move {
                serve_api(
                    api_listener,
                    node_cfg_str,
                    api_cfg,
                    node_clone,
                    inbox,
                    ws_events_rx,
                    Some(msg_encoder),
                )
                .await
                .expect("the REST API server should start successfully")
            }),
        );
    }

    let (hopr_socket, hopr_processes) = node.run(HoprServerReactor {}).await?;

    // process extracting the received data from the socket
    let mut ingress = hopr_socket.reader();
    processes.insert(
        HoprdProcesses::Socket,
        spawn(async move {
            while let Some(output) = ingress.next().await {
                match output {
                    TransportOutput::Received(data) => {
                        let recv_at = SystemTime::now();

                        // TODO: remove RLP in 3.0
                        match hopr_lib::rlp::decode(&data.plain_text) {
                            Ok((msg, sent)) => {
                                let latency = recv_at.as_unix_timestamp().saturating_sub(sent);

                                info!(
                                    app_tag = data.application_tag.unwrap_or(0),
                                    latency_in_ms = latency.as_millis(),
                                    "## NODE RECEIVED MESSAGE [@{}] ##",
                                    DateTime::<Utc>::from(recv_at).to_rfc3339(),
                                );

                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_MESSAGE_LATENCY.observe(latency.as_secs_f64());

                                if cfg.api.enable && ws_events_tx.receiver_count() > 0 {
                                    if let Err(e) =
                                        ws_events_tx.try_broadcast(TransportOutput::Received(ApplicationData {
                                            application_tag: data.application_tag,
                                            plain_text: msg.clone(),
                                        }))
                                    {
                                        error!("failed to notify websockets about a new message: {e}");
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
                                        "received a message with an ignored Inbox tag {:?}",
                                        data.application_tag
                                    )
                                }
                            }
                            Err(_) => error!("RLP decoding failed"),
                        }
                    }
                    TransportOutput::Sent(ack_challenge) => {
                        if cfg.api.enable && ws_events_tx.receiver_count() > 0 {
                            if let Err(e) = ws_events_tx.try_broadcast(TransportOutput::Sent(ack_challenge)) {
                                error!("failed to notify websockets about a new acknowledgement: {e}");
                            }
                        }
                    }
                }
            }
        }),
    );

    processes.extend(hopr_processes.into_iter().map(|(k, v)| (HoprdProcesses::HoprLib(k), v)));

    let mut signals = Signals::new([Signal::Hup, Signal::Int]).map_err(|e| HoprdError::OsError(e.to_string()))?;
    while let Some(Ok(signal)) = signals.next().await {
        match signal {
            Signal::Hup => {
                info!("Received the HUP signal... not doing anything");
            }
            Signal::Int => {
                info!("Received the INT signal... tearing down the node");
                futures::stream::iter(processes)
                    .for_each_concurrent(None, |(name, handle)| async move {
                        info!("Stopping process: {name:?}");
                        cancel_join_handle(handle).await
                    })
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
