use clap::Parser;
use hopr_lib::testing::fixtures::SWARM_N;

#[derive(Parser, Debug)]
#[command(name = "hopr-localcluster", about = "Run an in-process local HOPR cluster")]
pub struct Args {
    /// Number of nodes to start (max: SWARM_N)
    #[arg(long, default_value_t = SWARM_N)]
    pub size: usize,

    /// Channel funding amount in base units (per channel)
    #[arg(long, default_value = "1 wxHOPR")]
    pub funding_amount: String,

    /// Skip channel creation
    #[arg(long, default_value_t = false)]
    pub skip_channels: bool,

    /// REST API host to bind
    #[arg(long, default_value = "127.0.0.1")]
    pub api_host: String,

    /// REST API base port (node index is added)
    #[arg(long, default_value_t = 7000)]
    pub api_port_base: u16,

    /// Disable REST API authentication
    #[arg(long, default_value = "e2e-API-token^^")]
    pub api_token: String,
}
