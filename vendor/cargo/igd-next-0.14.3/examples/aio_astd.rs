//! IGD async API example.
//!
//! It demonstrates how to:
//! * get external IP
//! * add port mappings
//! * remove port mappings
//!
//! If everything works fine, 2 port mappings are added, 1 removed and we're left with single
//! port mapping: External 1234 ---> 4321 Internal

use std::env;
use std::net::SocketAddr;

use igd_next::aio::async_std::search_gateway;
use igd_next::PortMappingProtocol;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};

#[async_std::main]
async fn main() {
    let ip = match env::args().nth(1) {
        Some(ip) => ip,
        None => {
            println!("Local socket address is missing!");
            println!("This example requires a socket address representing the local machine and the port to bind to as an argument");
            println!("Example: target/[debug or release]/examples/aio 192.168.0.198:4321");
            println!("Example: cargo run --features aio_async_std --example aio_astd -- 192.168.0.198:4321");
            return;
        }
    };
    let ip: SocketAddr = ip.parse().expect("Invalid socket address");

    let _ = SimpleLogger::init(LevelFilter::Debug, LogConfig::default());

    let gateway = match search_gateway(Default::default()).await {
        Ok(g) => g,
        Err(err) => return println!("Faild to find IGD: {err}"),
    };
    let pub_ip = match gateway.get_external_ip().await {
        Ok(ip) => ip,
        Err(err) => return println!("Failed to get external IP: {err}"),
    };
    println!("Our public IP is {pub_ip}");
    if let Err(e) = gateway
        .add_port(PortMappingProtocol::TCP, ip.port(), ip, 120, "rust-igd-async-example")
        .await
    {
        println!("Failed to add port mapping: {e}");
    }
    println!("New port mapping was successfully added.");

    if let Err(e) = gateway
        .add_port(PortMappingProtocol::TCP, ip.port(), ip, 120, "rust-igd-async-example")
        .await
    {
        println!("Failed to add port mapping: {e}");
    }
    println!("New port mapping was successfully added.");

    if gateway.remove_port(PortMappingProtocol::TCP, ip.port()).await.is_err() {
        println!("Port mapping was not successfully removed");
    } else {
        println!("Port was removed.");
    }
}
