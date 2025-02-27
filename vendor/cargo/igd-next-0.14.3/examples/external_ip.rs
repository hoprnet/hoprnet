extern crate igd_next as igd;

fn main() {
    match igd::search_gateway(Default::default()) {
        Err(ref err) => println!("Error: {err}"),
        Ok(gateway) => match gateway.get_external_ip() {
            Err(ref err) => {
                println!("There was an error! {err}");
            }
            Ok(ext_addr) => {
                println!("Local gateway: {gateway}, External ip address: {ext_addr}");
            }
        },
    }
}
