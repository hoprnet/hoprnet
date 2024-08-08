extern crate igd_next as igd;
fn main() {
    match igd::search_gateway(Default::default()) {
        Err(ref err) => println!("Error: {err}"),
        Ok(gateway) => match gateway.remove_port(igd::PortMappingProtocol::TCP, 80) {
            Err(ref err) => {
                println!("There was an error! {err}");
            }
            Ok(()) => {
                println!("It worked");
            }
        },
    }
}
