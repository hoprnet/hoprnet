use std::{sync::Arc, time::SystemTime, path::Path, future::poll_fn, pin::Pin};

use async_lock::RwLock;
use chrono::{DateTime, Utc};

use futures::Stream;
use hoprd_api::run_hopr_api;
use hoprd_keypair::key_pair::{IdentityOptions, HoprKeys};
use hopr_lib::TransportOutput;
use utils_log::{info, warn};
use utils_types::traits::{PeerIdLike, ToHex};
use hoprd::cli::CliArgs;


const ONBOARDING_INFORMATION_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
    // // Metrics
    // TODO: introduce RLP
    // const metric_latency = create_histogram_with_buckets(
    //   'hoprd_histogram_message_latency_ms',
    //   'Histogram of measured received message latencies',
    //   new Float64Array([10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0])
    // )


#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("This is HOPRd {}", hopr_lib::constants::APP_VERSION);
    let args = <CliArgs as clap::Parser>::parse();

    // TOOD: add proper signal handling
    // The signal handling should produce the crossbeam-channel and notify all background loops to terminate gracefully
    // https://rust-cli.github.io/book/in-depth/signals.html

    if std::env::var("DAPPNODE").map(|v| v.to_lowercase() == "true").unwrap_or(false) {
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
        private_key: cfg.identity.private_key.clone().and_then(|v| hoprd::cli::parse_private_key(&v).ok()),
    };

    let hopr_keys = HoprKeys::init(identity_opts)?;

    info!("This node has packet key '{}' and uses a blockchain address '{}'",
        core_transport::Keypair::public(&hopr_keys.packet_key).to_peerid_str(),
        core_transport::Keypair::public(&hopr_keys.chain_key).to_hex()
    );

    // TODO: the following check can be removed once [PR](https://github.com/hoprnet/hoprnet/pull/5665) is merged
    if core_transport::Keypair::public(&hopr_keys.packet_key).to_string().starts_with("0xff") {
        warn!("This node uses an invalid packet key type and will not be able to become an effective relay node, please create a new identity!");
    }

    // Create the node instance
    info!("Creating the HOPRd node instance from hopr-lib");
    let hoprlib_cfg: hopr_lib::config::HoprLibConfig = cfg.clone().into();

    let mut node = hopr_lib::Hopr::new(
        hoprlib_cfg,
        &hopr_keys.packet_key,
        &hopr_keys.chain_key
    );

    let mut ingress = node.ingress();
    
    // Create the message inbox
    let inbox: Arc<RwLock<hoprd_inbox::Inbox>> = Arc::new(RwLock::new(hoprd_inbox::inbox::MessageInbox::new_with_time(cfg.inbox.clone(), || {
        platform::time::native::current_timestamp()
    })));
    
    let inbox_clone = inbox.clone();
    let node_ingress = async_std::task::spawn(async move {
        while let Some(output) = poll_fn(|cx| Pin::new(&mut ingress).poll_next(cx)).await {
            match output {
                TransportOutput::Received(msg) => {
                    let now = DateTime::<Utc>::from(SystemTime::now()).to_rfc3339();
                    info!("#### NODE RECEIVED MESSAGE [{now}] ####");
                    
                    // TODO: Move RLP for backwards compatibility to msg processor pipeline
                    //         let decodedMsg = decodeMessage(data.plain_text)
                    //         log(`Message: ${decodedMsg.msg}`)
                    //         log(`App tag: ${data.application_tag ?? 0}`)
                    //         log(`Latency: ${decodedMsg.latency} ms`)
                    //         metric_latency.observe(decodedMsg.latency)
                    
                    //         if (RPCH_MESSAGE_REGEXP.test(decodedMsg.msg)) {
                    //           log(`RPCh: received message [${decodedMsg.msg}]`)
                    //         }
                        
                        inbox_clone.write().await.push(msg).await;
                },
                TransportOutput::Sent(_ack_challenge) => {
                    // TODO: needed by the websockets 
                },
            }}
        }
    );
        
    let wait_til_end_of_time = node.run().await?;
        
    // Show onboarding information
    let my_address = core_transport::Keypair::public(&hopr_keys.chain_key).to_hex();
    let my_peer_id = core_transport::Keypair::public(&hopr_keys.packet_key).to_peerid();
    let version = hopr_lib::constants::APP_VERSION;
        
    while ! node.is_allowed_to_access_network(&my_peer_id).await {
        info!("
            Once you become eligible to join the HOPR network, you can continue your onboarding by using the following URL: https://hub.hoprnet.org/staking/onboarding?HOPRdNodeAddressForOnboarding={my_address}, or by manually entering the node address of your node on https://hub.hoprnet.org/.
        ");

        async_std::task::sleep(ONBOARDING_INFORMATION_INTERVAL).await;

        info!("
            Node information:

            Node peerID: {my_peer_id}
            Node address: {my_address}
            Node version: {version}
        ");
    }

    
    // setup API endpoint
    if cfg.api.enable {
        info!("Creating HOPRd API database");

        let hoprd_db_path = Path::new(&cfg.hopr.db.data).join("db").join("hoprd")
            .into_os_string()
            .into_string()
            .map_err(|_| hoprd::errors::HoprdError::FileError("failed to construct the HOPRd API database path".into()))?;
        
        // TODO: Authentication, needs implementing
        // if static token us used, this DB can be removed
        let hoprd_db = Arc::new(RwLock::new(hoprd::token::HoprdPersistentDb::new(utils_db::db::DB::new(
            utils_db::rusty::RustyLevelDbShim::new(&hoprd_db_path, true),
        ))));

        let host_listen = match &cfg.api.host.address {
            core_transport::config::HostType::IPv4(a) |
            core_transport::config::HostType::Domain(a) => format!("{a}:{}", cfg.api.host.port),
        };
        futures::join!(wait_til_end_of_time, node_ingress, run_hopr_api(&host_listen, &cfg.api, node, inbox.clone()));
    } else {
        futures::join!(wait_til_end_of_time, node_ingress);
    };

    Ok(())
}