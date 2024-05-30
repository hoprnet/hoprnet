use clap::{builder::ValueParser, Parser, Subcommand};
use hopr_lib::looks_like_domain;

fn parse_host(s: &str) -> Result<String, String> {
    let host = s.split_once(':').map_or(s, |(h, _)| h);
    if !(validator::ValidateIp::validate_ipv4(&host) || looks_like_domain(host)) {
        return Err(format!(
            "Given string {s} is not a valid host, should have a format: <ip>:<port> or <domain>(:<port>)"
        ));
    }
    Ok(s.to_owned())
}

/// # How to use it:
///
/// $ hopr-socks server --shost 127.0.0.1 --sport 1337
/// $ hopr-socks server --surl 127.0.0.1:1337
/// $ hopr-socks server --surl 127.0.0.1:1337 password --username admin --password password
///
#[derive(Debug, Parser)]
#[clap(name = "hopr_socks_server", about = "A simple SOCKS5 server implementation.")]
pub struct Opt {
    /// Socks server host
    #[clap(
        help = "Bind on address host",
        long = "shost",
        default_value = "127.0.0.1",
        global = true,
        value_parser = ValueParser::new(parse_host)
    )]
    pub socks_host: String,

    /// Socks server port
    #[clap(help = "Bind on address port", long = "sport", default_value = "1337", global = true)]
    pub socks_port: String,

    /// Socks server full address
    #[clap(
        help = "Full address to bind on (host + port)",
        long = "surl",
        global = true,
        value_parser = ValueParser::new(parse_host)
    )]
    pub socks_url: Option<String>,

    /// Request timeout
    #[clap(short = 't', long, default_value = "10")]
    pub request_timeout: u64,

    /// Choose authentication type
    #[clap(subcommand)]
    pub auth: Option<AuthMode>,
}

/// Choose the authentication type
#[derive(Subcommand, Debug)]
pub enum AuthMode {
    NoAuth,
    Password {
        #[clap(short, long)]
        username: String,

        #[clap(short, long)]
        password: String,
    },
}
impl Default for AuthMode {
    fn default() -> Self {
        Self::NoAuth
    }
}
