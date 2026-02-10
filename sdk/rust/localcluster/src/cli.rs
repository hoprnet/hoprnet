use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "hopr-localcluster",
    about = "Run a local HOPR cluster using external processes"
)]
pub struct Args {
    /// Number of nodes to start
    #[arg(long, default_value_t = 5)]
    pub size: usize,

    /// Channel funding amount in base units (per channel)
    #[arg(long, default_value = "1 wxHOPR")]
    pub funding_amount: String,

    /// Skip channel creation
    #[arg(long, default_value_t = false)]
    pub skip_channels: bool,

    /// REST API host to bind
    #[arg(long, default_value = "localhost")]
    pub api_host: String,

    /// REST API base port (node index is added)
    #[arg(long, default_value_t = 3000)]
    pub api_port_base: u16,

    /// P2P host to bind
    #[arg(long, default_value = "localhost")]
    pub p2p_host: String,

    /// P2P base port (node index is added)
    #[arg(long, default_value_t = 9000)]
    pub p2p_port_base: u16,

    /// Base directory for generated configs, identities, DBs, and logs
    #[arg(long, default_value = "/tmp/hopr-localcluster")]
    pub data_dir: String,

    /// Docker image containing both Anvil and Blokli
    #[arg(long)]
    pub chain_image: String,

    /// Path to the hoprd binary
    #[arg(long, default_value = "hoprd")]
    pub hoprd_bin: String,

    /// Path to the hoprd-gen-test binary
    #[arg(long, default_value = "hoprd-gen-test")]
    pub hoprd_gen_test_bin: String,

    /// Password used to encrypt identities
    #[arg(long, default_value = "password")]
    pub identity_password: String,

    /// API token for hoprd REST API (enables authentication)
    #[arg(long)]
    pub api_token: Option<String>,
}
