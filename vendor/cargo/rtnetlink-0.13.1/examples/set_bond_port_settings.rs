// SPDX-License-Identifier: MIT

use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};
use std::env;

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        return Ok(());
    }
    let link_name = &args[1];

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    set_bond_port_settings(handle, link_name.to_string())
        .await
        .map_err(|e| format!("{e}"))
}
// The bond port priority is only supported to set when bonding mode is
// active-backup(1) or balance-tlb (5) or balance-alb (6)
async fn set_bond_port_settings(
    handle: Handle,
    name: String,
) -> Result<(), Error> {
    let mut links = handle.link().get().match_name(name.clone()).execute();
    if let Some(link) = links.try_next().await? {
        // This is equivalent to `ip link set name NAME type bond_slave queue_id
        // 0 prio 1`. The port priority setting is supported in kernel
        // since v6.0
        handle
            .link()
            .set_bond_port(link.header.index)
            .queue_id(0)
            .prio(1)
            .execute()
            .await?
    } else {
        println!("no link link {name} found");
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "usage:
    cargo run --example set_bond_port_settings -- <link name>

Note that you need to run this program as root. Instead of running cargo as root,
build the example normally:

    cd netlink-ip ; cargo build --example set_bond_port_settings

Then find the binary in the target directory:

    cd ../target/debug/example ; sudo ./set_bond_port_settings <link_name>"
    );
}
