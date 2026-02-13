use anyhow::Context;
use clap::Parser;
use hopr_chain_connector::{
    BlockchainConnectorConfig,
    api::*,
    blokli_client::{BlokliClient, BlokliClientConfig, BlokliQueryClient},
    create_trustful_safeless_hopr_blokli_connector,
    reexports::hopr_chain_types::exports::alloy::hex,
};
use hopr_lib::{ChainKeypair, HoprKeys, Keypair, SafeModule, XDaiBalance, crypto_traits::Randomizable};
use hopr_reference::config::SessionIpForwardingConfig;
use hoprd::config::{Db, HoprdConfig, Identity, UserHoprLibConfig};
use hoprd_api::config::{Api, Auth};

/// Tool used to generate test node Safes and hoprd configuration files.
///
/// This tool generates nodes identities, deploys and funds its Safes, and generates node
/// configuration files to be used with `hoprd`.
///
/// This is mostly useful for testing purposes.
#[derive(Parser, Debug)]
#[command(name = "hoprd-gen-test", author, version, about = "Tool used to generate test node Safes and hoprd configuration files", long_about = None)]
struct Args {
    /// Blokli URL
    #[arg(long, short, default_value = "http://localhost:8080")]
    blokli_url: String,

    /// Private key of the Smart Contract deployer
    #[arg(
        long,
        short,
        default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    private_key: String,

    /// Number of nodes to generate.
    #[arg(long, short, default_value = "3")]
    num_nodes: usize,

    /// Home path where all the node data (config, identity, db) will be stored.
    #[arg(long, short, default_value = "/tmp/hopr-nodes")]
    config_home: String,

    #[arg(long, default_value = "password")]
    identity_password: String,

    /// Whether to generate random IDs or fixed deterministic ones.
    #[arg(long, short, default_value = "false")]
    random_identities: bool,
}

lazy_static::lazy_static! {
    static ref NODE_KEYS: [HoprKeys; 5] = [
        (
            hex!("76a4edbc3f595d4d07671779a0055e30b2b8477ecfd5d23c37afd7b5aa83781d"),
            hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a")
        ).try_into().unwrap(),
        (
            hex!("c90f09e849aa512be3dd007452977e32c7cfdc1e3de1a62bd92ba6592bcc9e90"),
            hex!("c3659450e994f3ad086373440e4e7070629a1bfbd555387237ccb28d17acbfc8")
        ).try_into().unwrap(),
        (
            hex!("40d4749a620d1a4278d030a3153b5b94d6fcd4f9677f6ce8e37e6ebb1987ad53"),
            hex!("4a14c5aeb53629a2dd45058a8d233f24dd90192189e8200a1e5f10069868f963")
        ).try_into().unwrap(),
        (
            hex!("e539f1ac48270be4e84b6acfe35252df5e141a29b50ddb07b50670271bb574ee"),
            hex!("8c1edcdebfe508031e4124168bb4a133180e8ee68207a7946fcdc4ad0068ef0d")
        ).try_into().unwrap(),
        (
            hex!("9ab557eb14d8b081c7e1750eb87407d8c421aa79bdeb420f38980829e7dbf936"),
            hex!("6075c595103667537c33cdb954e3e5189921cab942e5fc0ba9ec27fe6d7787d1")
        ).try_into().unwrap()
    ];
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    std::fs::create_dir_all(&args.config_home)?;
    let home_path = std::path::Path::new(&args.config_home);
    let private_key = hex::decode(&args.private_key).context("invalid private key")?;

    let blokli_client = BlokliClient::new(args.blokli_url.parse()?, BlokliClientConfig::default());
    let status = blokli_client.query_health().await?;
    if !status.eq_ignore_ascii_case("ok") {
        return Err(anyhow::anyhow!("Blokli is not usable: {status}"));
    }

    // Create connector for the deployer account
    let mut anvil_connector = create_trustful_safeless_hopr_blokli_connector(
        &ChainKeypair::from_secret(&private_key)?,
        Default::default(),
        blokli_client.clone(),
    )
    .await?;
    anvil_connector.connect().await?;

    let initial_token_balance: HoprBalance = "1000 wxHOPR".parse()?;
    let initial_native_balance: XDaiBalance = "1 xDai".parse()?;

    for id in 0..args.num_nodes.clamp(1, NODE_KEYS.len()) {
        let kp = if args.random_identities {
            HoprKeys::random()
        } else {
            NODE_KEYS[id].clone()
        };
        let node_address = kp.chain_key.public().to_address();
        eprintln!("Node {id}: Address {node_address}");

        let node_connector = std::sync::Arc::new(
            create_trustful_safeless_hopr_blokli_connector(
                &kp.chain_key,
                BlockchainConnectorConfig::default(),
                blokli_client.clone(),
            )
            .await?,
        );

        eprint!("Node {id}: Checking balances...");

        // Send 1 xDai to the new node address from Anvil 0 account
        let node_native_balance: XDaiBalance = node_connector.balance(node_address).await?;
        if node_native_balance < initial_native_balance {
            let top_up = initial_native_balance - node_native_balance;
            if anvil_connector.balance(*anvil_connector.me()).await? < top_up {
                return Err(anyhow::anyhow!(
                    "Account {} must have at least {top_up}.",
                    anvil_connector.me()
                ));
            }

            anvil_connector.withdraw(top_up, &node_address).await?.await?;
            eprint!("\x1b[2K\rNode {id}: {top_up} transferred to {node_address}");
        } else {
            eprint!("\x1b[2K\rNode {id}: {node_address} already has {node_native_balance} xDai tokens");
        }

        eprint!("\x1b[2K\rNode {id}: Checking Safe deployment...");
        let safe = if let Some(safe) = node_connector.safe_info(SafeSelector::Owner(node_address)).await? {
            safe
        } else {
            // Send 1000 wxHOPR tokens to the new node address from Anvil 0 account
            eprint!("\x1b[2K\rNode {id}: Topping up to {initial_token_balance}...");
            let node_token_balance: HoprBalance = node_connector.balance(node_address).await?;
            if node_token_balance < initial_token_balance {
                let top_up = initial_token_balance - node_token_balance;
                if anvil_connector.balance(*anvil_connector.me()).await? < top_up {
                    return Err(anyhow::anyhow!(
                        "Account {} must have at least {top_up}.",
                        anvil_connector.me()
                    ));
                }

                anvil_connector.withdraw(top_up, &node_address).await?.await?;
                eprint!("\x1b[2K\rNode {id}: {top_up} transferred to {node_address}");
            } else {
                eprint!("\x1b[2K\rNode {id}: {node_address} already has {node_token_balance} wxHOPR tokens");
            }

            eprint!("\x1b[2K\rNode {id}: Deploying Safe...");
            let node_connector_clone = node_connector.clone();
            let jh = tokio::task::spawn(async move {
                node_connector_clone
                    .await_safe_deployment(SafeSelector::Owner(node_address), std::time::Duration::from_secs(10))
                    .await
            });
            node_connector.deploy_safe(initial_token_balance).await?.await?;
            jh.await??
        };

        let id_file = home_path
            .join(format!("node_id_{id}.id"))
            .to_str()
            .ok_or(anyhow::anyhow!("Invalid path"))?
            .to_owned();

        let node_cfg = HoprdConfig {
            hopr: UserHoprLibConfig {
                announce: true,
                safe_module: SafeModule {
                    safe_address: safe.address,
                    module_address: safe.module,
                },
                ..Default::default()
            },
            identity: Identity {
                file: id_file.clone(),
                password: args.identity_password.clone(),
                private_key: None,
            },
            db: Db {
                data: home_path
                    .join(format!("db_{id}"))
                    .to_str()
                    .ok_or(anyhow::anyhow!("Invalid path"))?
                    .to_owned(),
                initialize: true,
                force_initialize: true,
            },
            api: Api {
                enable: true,
                auth: Auth::None,
                ..Default::default()
            },
            blokli_url: Some(args.blokli_url.clone()),
            session_ip_forwarding: SessionIpForwardingConfig {
                use_target_allow_list: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let cfg_file = home_path
            .join(format!("hoprd_cfg_{id}.yaml"))
            .to_str()
            .ok_or(anyhow::anyhow!("Invalid path"))?
            .to_owned();
        std::fs::write(&cfg_file, serde_yaml::to_string(&node_cfg)?)?;
        kp.write_eth_keystore(&id_file, &args.identity_password)?;

        eprintln!("\x1b[2K\rNode {id}: Node config written to {cfg_file}");
    }

    Ok(())
}
