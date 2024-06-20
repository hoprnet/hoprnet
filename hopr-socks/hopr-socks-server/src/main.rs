use clap::Parser;
use fast_socks5::Result;
use hopr_socks_server::cli::Opt;
use hopr_socks_server::SocksServer;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let subscriber = tracing_subscriber::Registry::default().with(env_filter).with(format);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let opt: Opt = Opt::parse();
    let socks_domain = match opt.url {
        Some(url) => url,
        None => [opt.host.clone(), opt.port.clone()].join(":"),
    };

    let server = SocksServer::new(socks_domain, opt.request_timeout, opt.auth.unwrap_or_default()).await?;

    server.run().await
}
