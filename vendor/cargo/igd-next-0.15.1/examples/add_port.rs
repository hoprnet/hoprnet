use std::net::{IpAddr, SocketAddr};

extern crate igd_next as igd;

fn main() {
    match igd::search_gateway(Default::default()) {
        Err(ref err) => println!("Error: {err}"),
        Ok(gateway) => {
            let local_addr = match std::env::args().nth(1) {
                Some(local_addr) => local_addr,
                None => panic!("Expected IP address (cargo run --example add_port <your IP here>)"),
            };
            let local_addr = local_addr.parse::<IpAddr>().unwrap();
            let local_addr = SocketAddr::new(local_addr, 8080u16);

            match gateway.add_port(igd::PortMappingProtocol::TCP, 80, local_addr, 60, "add_port example") {
                Err(ref err) => {
                    println!("There was an error! {err}");
                }
                Ok(()) => {
                    println!("It worked");
                }
            }
        }
    }
}
