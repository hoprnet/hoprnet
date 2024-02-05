use std::str::FromStr;
use std::{future::poll_fn, pin::Pin, sync::Arc, time::SystemTime};

use async_lock::RwLock;
use chrono::{DateTime, Utc};

use futures::Stream;
use hopr_lib::{ApplicationData, AsUnixTimestamp, ToHex, TransportOutput};
use hoprd::cli::CliArgs;
use hoprd_api::run_hopr_api;
use hoprd_keypair::key_pair::{HoprKeys, IdentityOptions};
use log::{error, info, warn};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleHistogram;

const ONBOARDING_INFORMATION_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
const WEBSOCKET_EVENT_BROADCAST_CAPACITY: usize = 10000;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_MESSAGE_LATENCY: SimpleHistogram = SimpleHistogram::new(
        "hopr_message_latency_sec",
        "Histogram of measured received message latencies in seconds",
        vec![0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0]
    ).unwrap();
}

fn setup_logger(level: log::LevelFilter) {
    if let Err(e) = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .level_for("libp2p_mplex", log::LevelFilter::Info)
        .level_for("multistream_select", log::LevelFilter::Info)
        .level_for("sqlx::query", log::LevelFilter::Info)
        .level_for("tracing::span", log::LevelFilter::Error)
        .level_for("isahc::handler", log::LevelFilter::Error)
        .level_for("isahc::client", log::LevelFilter::Error)
        .level_for("surf::middleware::logger::native", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .apply()
    {
        eprintln!("failed to setup logger: {e}")
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger(
        std::env::var("RUST_LOG")
            .map_err(|_| ())
            .and_then(|level| log::LevelFilter::from_str(&level).map_err(|_| ()))
            .unwrap_or(log::LevelFilter::Info),
    );

    info!("This is HOPRd {}", hopr_lib::constants::APP_VERSION);
    let args = <CliArgs as clap::Parser>::parse();

    // TOOD: add proper signal handling
    // The signal handling should produce the crossbeam-channel and notify all background loops to terminate gracefully
    // https://rust-cli.github.io/book/in-depth/signals.html

    if std::env::var("DAPPNODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        info!("The HOPRd node appears to run on DappNode");
    }

    let cfg = hoprd::config::HoprdConfig::from_cli_args(args, false)?;
    info!("Node configuration: {}", cfg.as_redacted_string()?);

    if let hopr_lib::HostType::IPv4(address) = &cfg.hopr.host.address {
        let ipv4 = std::net::Ipv4Addr::from_str(address)?;

        if ipv4.is_loopback() && !cfg.hopr.transport.announce_local_addresses {
            return Err(hopr_lib::errors::HoprLibError::GeneralError(
                "Cannot announce a loopback address".into(),
            ))?;
        }
    }

    // Find or create an identity
    let identity_opts = IdentityOptions {
        initialize: cfg.hopr.db.initialize,
        id_path: cfg.identity.file.clone(),
        password: cfg.identity.password.clone(),
        private_key: cfg
            .identity
            .private_key
            .clone()
            .and_then(|v| hoprd::cli::parse_private_key(&v).ok()),
    };

    let hopr_keys = HoprKeys::init(identity_opts)?;

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

    let mut node = hopr_lib::Hopr::new(hoprlib_cfg, &hopr_keys.packet_key, &hopr_keys.chain_key);
    let mut ingress = node.ingress();

    let node = Arc::new(node);

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
    let node_ingress = async_std::task::spawn(async move {
        while let Some(output) = poll_fn(|cx| Pin::new(&mut ingress).poll_next(cx)).await {
            match output {
                TransportOutput::Received(data) => {
                    let recv_at = SystemTime::now();

                    // TODO: remove RLP in 3.0
                    match hopr_lib::rlp::decode(&data.plain_text) {
                        Ok((msg, sent)) => {
                            let latency = recv_at.duration_since(SystemTime::UNIX_EPOCH).unwrap() - sent;

                            info!(
                                r#"
                            #### NODE RECEIVED MESSAGE [{}] ####
                            Message: {}
                            App tag: {}
                            Latency: {}ms"#,
                                DateTime::<Utc>::from(recv_at).to_rfc3339(),
                                String::from_utf8_lossy(&msg),
                                data.application_tag.unwrap_or(0),
                                latency.as_millis()
                            );

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_MESSAGE_LATENCY.observe(latency.as_secs_f64());

                            if cfg.api.enable && ws_events_tx.receiver_count() > 0 {
                                if let Err(e) = ws_events_tx.try_broadcast(TransportOutput::Received(ApplicationData {
                                    application_tag: data.application_tag,
                                    plain_text: msg.clone(),
                                })) {
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
    });

    let hopr_clone = node.clone();
    let run_the_hopr_node = async move {
        // start indexing and initiate the node
        let processes = hopr_clone.run().await.expect("the HOPR node should run without errors");

        // Show onboarding information
        let my_ethereum_address = hopr_lib::Keypair::public(&hopr_keys.chain_key).to_address().to_hex();
        let my_peer_id = (*hopr_lib::Keypair::public(&hopr_keys.packet_key)).into();
        let version = hopr_lib::constants::APP_VERSION;

        while !hopr_clone.is_allowed_to_access_network(&my_peer_id).await {
            info!("
                Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding={my_ethereum_address}, or by manually entering the node address of your node on https://hub.hoprnet.org/.
            ");

            async_std::task::sleep(ONBOARDING_INFORMATION_INTERVAL).await;

            info!(
                "
                Node information:

                Node peerID: {my_peer_id}
                Node Ethereum address: {my_ethereum_address} <- put this into staking hub
                Node version: {version}
            "
            );
        }

        // now that the node is allowed in the network, run the background processes
        processes.await;
    };

    // setup API endpoint
    if cfg.api.enable {
        info!("Running HOPRd with the API...");

        // TODO: remove RLP in 3.0
        let msg_encoder =
            |data: &[u8]| hopr_lib::rlp::encode(data, hopr_platform::time::native::current_time().as_unix_timestamp());

        let host_listen = match &cfg.api.host.address {
            hopr_lib::HostType::IPv4(a) | hopr_lib::HostType::Domain(a) => {
                format!("{a}:{}", cfg.api.host.port)
            }
        };

        futures::join!(
            run_the_hopr_node,
            node_ingress,
            run_hopr_api(
                &host_listen,
                &cfg.api,
                node,
                inbox.clone(),
                ws_events_rx,
                Some(msg_encoder)
            )
        );
    } else {
        info!("Running HOPRd without the API...");

        futures::join!(run_the_hopr_node, node_ingress);
    };

    Ok(())
}
