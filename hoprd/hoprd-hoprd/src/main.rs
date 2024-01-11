use std::str::FromStr;
use std::{future::poll_fn, pin::Pin, sync::Arc, time::SystemTime};

use async_lock::RwLock;
use chrono::{DateTime, Utc};

use futures::Stream;
use hopr_lib::{ApplicationData, TransportOutput};
use hoprd::cli::CliArgs;
use hoprd_api::run_hopr_api;
use hoprd_keypair::key_pair::{HoprKeys, IdentityOptions};
use log::{error, info, warn};
use utils_types::traits::{PeerIdLike, ToHex};

#[cfg(all(feature = "prometheus", not(test)))]
use metrics::metrics::SimpleHistogram;

const ONBOARDING_INFORMATION_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_MESSAGE_LATENCY: SimpleHistogram = SimpleHistogram::new(
        "hoprd_histogram_message_latency_ms",
        "Histogram of measured received message latencies in milliseconds",
        vec![10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0]
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
            .unwrap_or(log::LevelFilter::Debug),
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

    // Find or create an identity
    let identity_opts = IdentityOptions {
        initialize: cfg.hopr.db.initialize,
        id_path: cfg.identity.file.clone(),
        password: cfg.identity.password.clone(),
        use_weak_crypto: Some(cfg.test.use_weak_crypto),
        private_key: cfg
            .identity
            .private_key
            .clone()
            .and_then(|v| hoprd::cli::parse_private_key(&v).ok()),
    };

    let hopr_keys = HoprKeys::init(identity_opts)?;

    info!(
        "This node has packet key '{}' and uses a blockchain address '{}'",
        core_transport::Keypair::public(&hopr_keys.packet_key).to_peerid_str(),
        core_transport::Keypair::public(&hopr_keys.chain_key).to_hex()
    );

    // TODO: the following check can be removed once [PR](https://github.com/hoprnet/hoprnet/pull/5665) is merged
    if core_transport::Keypair::public(&hopr_keys.packet_key)
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

    // Create the message inbox
    let inbox: Arc<RwLock<hoprd_inbox::Inbox>> = Arc::new(RwLock::new(
        hoprd_inbox::inbox::MessageInbox::new_with_time(cfg.inbox.clone(), || {
            platform::time::native::current_timestamp()
        }),
    ));

    let inbox_clone = inbox.clone();
    let node_ingress = async_std::task::spawn(async move {
        while let Some(output) = poll_fn(|cx| Pin::new(&mut ingress).poll_next(cx)).await {
            match output {
                TransportOutput::Received(data) => {
                    let recv_at = SystemTime::now();

                    // TODO: remove RLP in 3.0
                    match utils_types::rlp::decode(&data.plain_text) {
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
                            METRIC_MESSAGE_LATENCY.observe(latency.as_millis() as f64);

                            inbox_clone
                                .write()
                                .await
                                .push(ApplicationData {
                                    application_tag: data.application_tag,
                                    plain_text: msg,
                                })
                                .await;
                        }
                        Err(_) => error!("RLP decoding failed"),
                    }
                }
                TransportOutput::Sent(_ack_challenge) => {
                    // TODO: needed by the websockets
                }
            }
        }
    });

    let wait_til_end_of_time = node.run().await?;

    // Show onboarding information
    let my_address = core_transport::Keypair::public(&hopr_keys.chain_key).to_hex();
    let my_peer_id = core_transport::Keypair::public(&hopr_keys.packet_key).to_peerid();
    let version = hopr_lib::constants::APP_VERSION;

    while !node.is_allowed_to_access_network(&my_peer_id).await {
        info!("
            Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding={my_address}, or by manually entering the node address of your node on https://hub.hoprnet.org/.
        ");

        async_std::task::sleep(ONBOARDING_INFORMATION_INTERVAL).await;

        info!(
            "
            Node information:

            Node peerID: {my_peer_id}
            Node address: {my_address}
            Node version: {version}
        "
        );
    }

    // setup API endpoint
    if cfg.api.enable {
        info!("Running HOPRd with the API...");

        // TODO: remove RLP in 3.0
        let msg_encoder = |data: &[u8]| utils_types::rlp::encode(data, platform::time::native::current_timestamp());

        let host_listen = match &cfg.api.host.address {
            core_transport::config::HostType::IPv4(a) | core_transport::config::HostType::Domain(a) => {
                format!("{a}:{}", cfg.api.host.port)
            }
        };

        futures::join!(
            wait_til_end_of_time,
            node_ingress,
            run_hopr_api(&host_listen, &cfg.api, node, inbox.clone(), Some(msg_encoder))
        );
    } else {
        info!("Running HOPRd without the API...");

        futures::join!(wait_til_end_of_time, node_ingress);
    };

    Ok(())
}
